#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import signal
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parents[2]
STATE_FILE = ROOT / "demo-artifacts/pids/vscode-window.json"
DEFAULT_LAUNCHER = Path("/home/ops/dev/vnc/vscode-recording/launch-code-recording.sh")
DEFAULT_PROFILE_DIR = Path("/home/ops/dev/vnc/vscode-recording/profile")


def load_env() -> None:
    initial = set(os.environ)

    def load_file(path: Path, *, override_loaded: bool = False) -> None:
        if not path.is_file():
            return
        for raw in path.read_text(errors="ignore").splitlines():
            line = raw.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, value = line.split("=", 1)
            key = key.strip()
            if not key.replace("_", "").isalnum() or key[0].isdigit():
                continue
            if key not in os.environ or (override_loaded and key not in initial):
                os.environ[key] = value.strip().strip('"').strip("'")

    for path in (ROOT / ".env.demo.local", ROOT / "scripts/demo/.env.demo.local"):
        if path.is_file():
            load_file(path)
            break
    load_file(ROOT / "demo-artifacts/display.env", override_loaded=True)


def env() -> dict[str, str]:
    data = os.environ.copy()
    data.setdefault("DEMO_DISPLAY", ":96")
    data.setdefault("DISPLAY", data["DEMO_DISPLAY"])
    data.setdefault("DEMO_SCREEN_SIZE", "1920x1080")
    return data


def run(cmd: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, env=env(), text=True, capture_output=True, check=check)


def ensure_recording_profile(*, reset_state: bool = False) -> None:
    profile_dir = Path(os.environ.get("DEMO_VSCODE_PROFILE_DIR", str(DEFAULT_PROFILE_DIR)))
    if reset_state:
        for relative in ("Backups", "User/workspaceStorage", "User/History"):
            shutil.rmtree(profile_dir / relative, ignore_errors=True)
    user_dir = profile_dir / "User"
    user_dir.mkdir(parents=True, exist_ok=True)

    settings_path = user_dir / "settings.json"
    try:
        settings = json.loads(settings_path.read_text()) if settings_path.is_file() else {}
    except json.JSONDecodeError:
        settings = {}
    excluded_tree_paths = {
        "**/.claude": True,
        "**/.cloud": True,
        "**/.clickhouse": True,
        "**/.github": True,
        "**/.venv": True,
        "**/.venv-demo": True,
        "**/.vscode": True,
        "**/__pycache__": True,
        "**/demo-artifacts": True,
        "**/node_modules": True,
        "**/target": True,
        "**/.env": True,
        "**/.env.demo.local": True,
    }
    files_exclude = settings.get("files.exclude")
    if not isinstance(files_exclude, dict):
        files_exclude = {}
    files_exclude.update(excluded_tree_paths)
    settings.update(
        {
            "explorer.autoReveal": True,
            "explorer.compactFolders": False,
            "explorer.decorations.badges": False,
            "explorer.decorations.colors": False,
            "files.exclude": files_exclude,
            "files.hotExit": "off",
            "git.decorations.enabled": False,
            "git.enabled": False,
            "scm.countBadge": "off",
            "scm.diffDecorations": "none",
            "workbench.editor.confirmRevert": False,
            "workbench.editor.enablePreview": False,
            "workbench.editor.showTabs": "multiple",
            "workbench.sideBar.location": "left",
            "workbench.startupEditor": "none",
        }
    )
    settings_path.write_text(json.dumps(settings, indent=2) + "\n")

    keybindings_path = user_dir / "keybindings.json"
    try:
        keybindings = json.loads(keybindings_path.read_text()) if keybindings_path.is_file() else []
    except json.JSONDecodeError:
        keybindings = []
    required = [
        {"key": "ctrl+alt+shift+e", "command": "workbench.action.closeAuxiliaryBar"},
        {"key": "ctrl+alt+shift+n", "command": "notifications.hideToasts"},
        {"key": "ctrl+alt+shift+l", "command": "notifications.hideList"},
        {"key": "ctrl+alt+shift+r", "command": "workbench.action.files.revert"},
        {"key": "ctrl+alt+shift+w", "command": "workbench.action.closeAllEditors"},
        {"key": "ctrl+alt+shift+c", "command": "workbench.files.action.collapseExplorerFolders"},
        {"key": "ctrl+shift+e", "command": "workbench.view.explorer"},
    ]
    existing = {(str(item.get("key")), str(item.get("command"))) for item in keybindings if isinstance(item, dict)}
    for item in required:
        identity = (item["key"], item["command"])
        if identity not in existing:
            keybindings.append(item)
    keybindings_path.write_text(json.dumps(keybindings, indent=2) + "\n")


def runbook() -> dict[str, Any]:
    return yaml.safe_load((ROOT / "scripts/demo/runbook.yaml").read_text())


def scene_by_id(scene_id: str) -> dict[str, Any]:
    for scene in runbook()["scenes"]:
        if scene["id"] == scene_id:
            return scene
    raise KeyError(scene_id)


def xdo(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return run(["xdotool", *args], check=check)


def screen_frame() -> tuple[int, int, int, int]:
    screen_w, screen_h = [int(part) for part in env().get("DEMO_SCREEN_SIZE", "1920x1080").split("x", 1)]
    margin = int(os.environ.get("DEMO_VSCODE_MARGIN", "56"))
    return margin, margin, screen_w - (margin * 2), screen_h - (margin * 2)


def window_geometry(window_id: str) -> tuple[int, int, int, int] | None:
    result = xdo("getwindowgeometry", "--shell", window_id, check=False)
    if result.returncode != 0:
        return None
    values: dict[str, int] = {}
    for line in result.stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        if key in {"X", "Y", "WIDTH", "HEIGHT"}:
            values[key] = int(value)
    if {"X", "Y", "WIDTH", "HEIGHT"} <= values.keys():
        return values["X"], values["Y"], values["WIDTH"], values["HEIGHT"]
    return None


def move_window(window_id: str) -> None:
    x, y, width, height = screen_frame()
    subprocess.run(["wmctrl", "-ir", window_id, "-e", f"0,{x},{y},{width},{height}"], cwd=ROOT, env=env(), check=False)
    xdo("windowmove", window_id, str(x), str(y), check=False)
    xdo("windowsize", window_id, str(width), str(height), check=False)
    xdo("windowraise", window_id, check=False)
    xdo("windowfocus", window_id, check=False)
    hide_mouse()


def focus_explorer(window_id: str) -> None:
    xdo("windowraise", window_id, check=False)
    xdo("windowfocus", window_id, check=False)
    for key in ("ctrl+alt+shift+e", "ctrl+alt+shift+n", "ctrl+alt+shift+l", "Escape"):
        xdo("key", key, check=False)
        time.sleep(0.08)
    xdo("key", "ctrl+shift+e", check=False)
    time.sleep(0.2)
    hide_mouse()


def focus_editor(window_id: str) -> None:
    xdo("windowraise", window_id, check=False)
    xdo("windowfocus", window_id, check=False)
    xdo("key", "ctrl+1", check=False)
    time.sleep(0.12)
    hide_mouse()


def collapse_explorer(window_id: str) -> None:
    focus_explorer(window_id)
    xdo("key", "ctrl+alt+shift+c", check=False)
    time.sleep(0.35)
    xdo("key", "Home", check=False)
    time.sleep(0.08)
    xdo("key", "Right", check=False)
    time.sleep(0.2)
    hide_mouse()


def close_all_editors(window_id: str) -> None:
    xdo("windowraise", window_id, check=False)
    xdo("windowfocus", window_id, check=False)
    xdo("key", "ctrl+alt+shift+w", check=False)
    time.sleep(0.25)
    collapse_explorer(window_id)


def hide_mouse() -> None:
    screen_w, screen_h = [int(part) for part in env().get("DEMO_SCREEN_SIZE", "1920x1080").split("x", 1)]
    xdo("mousemove", str(screen_w - 2), str(screen_h - 2), check=False)


def code_windows() -> list[tuple[str, str]]:
    result = xdo("search", "--class", "Code", check=False)
    windows: list[tuple[str, str]] = []
    for raw in result.stdout.splitlines():
        window_id = raw.strip()
        if not window_id.isdigit():
            continue
        title = xdo("getwindowname", window_id, check=False).stdout.strip()
        geometry = window_geometry(window_id)
        if geometry is None:
            continue
        _, _, width, height = geometry
        if width < 200 or height < 200:
            continue
        windows.append((window_id, title))
    return windows


def wait_for_code_window(expected_name: str | None = None) -> str:
    expected = (expected_name or "").lower()
    fallback: str | None = None
    for _ in range(100):
        for window_id, title in code_windows():
            if fallback is None:
                fallback = window_id
            title_lower = title.lower()
            if not expected or expected in title_lower:
                return window_id
        time.sleep(0.1)
    if fallback:
        return fallback
    raise RuntimeError("VS Code window did not appear")


def launcher_path(*, reset_state: bool = False) -> Path:
    ensure_recording_profile(reset_state=reset_state)
    configured = os.environ.get("DEMO_VSCODE_LAUNCHER")
    launcher = Path(configured) if configured else DEFAULT_LAUNCHER
    if not launcher.is_file():
        raise FileNotFoundError(
            f"VS Code recording launcher not found: {launcher}. "
            "Run the desktop VS Code setup before recording VS Code scenes."
        )
    return launcher


def open_workspace() -> str:
    command = [str(launcher_path(reset_state=True)), "--new-window", str(ROOT)]
    result = subprocess.run(command, cwd=ROOT, env=env(), text=True, capture_output=True)
    if result.returncode != 0:
        raise RuntimeError(result.stdout + result.stderr)
    window_id = wait_for_code_window(ROOT.name)
    move_window(window_id)
    focus_explorer(window_id)
    close_all_editors(window_id)
    write_state(window_id)
    return window_id


def open_file_at_line(relative: str, line: int) -> str:
    path = ROOT / relative
    if not path.is_file():
        raise FileNotFoundError(path)
    command = [
        str(launcher_path()),
        "--reuse-window",
        "--goto",
        f"{path}:{line}:1",
    ]
    result = subprocess.run(command, cwd=ROOT, env=env(), text=True, capture_output=True)
    if result.returncode != 0:
        raise RuntimeError(result.stdout + result.stderr)
    window_id = wait_for_code_window(path.name)
    move_window(window_id)
    write_state(window_id)
    hide_mouse()
    return window_id


def first_steps_by_file(steps: list[dict[str, Any]]) -> list[dict[str, Any]]:
    seen: set[str] = set()
    unique: list[dict[str, Any]] = []
    for step in steps:
        relative = str(step["file"])
        if relative in seen:
            continue
        seen.add(relative)
        unique.append(step)
    return unique


def preload_tabs(window_id: str, steps: list[dict[str, Any]]) -> str:
    for step in first_steps_by_file(steps):
        window_id = open_file_at_line(str(step["file"]), int(step.get("line", 1)))
        time.sleep(float(os.environ.get("DEMO_VSCODE_PRELOAD_FILE_PAUSE_SECONDS", "0.22")))
    first = steps[0]
    window_id = open_file_at_line(str(first["file"]), int(first.get("line", 1)))
    focus_explorer(window_id)
    return window_id


def show_file(window_id: str, step: dict[str, Any]) -> str:
    relative = str(step["file"])
    line = int(step.get("line", 1))
    window_id = open_file_at_line(relative, line)
    if "edit_line" in step:
        edit_current_line(window_id, str(step["edit_line"]), step)
    elif step.get("uncomment_and_set_true"):
        uncomment_and_set_true_current_line(window_id, step)
    else:
        focus_explorer(window_id)
    print(f"shown {relative}:{line}")
    return window_id


def edited_runbook_steps() -> list[dict[str, Any]]:
    steps: list[dict[str, Any]] = []
    for scene in runbook()["scenes"]:
        human = scene.get("human") or {}
        if human.get("type") != "vscode":
            continue
        for step in human.get("steps") or []:
            if ("edit_line" in step or step.get("uncomment_and_set_true")) and "file" in step:
                steps.append(step)
    return steps


def edit_current_line(window_id: str, replacement: str, step: dict[str, Any]) -> None:
    focus_editor(window_id)
    time.sleep(float(step.get("pause_before_edit", 0.5)))
    xdo("key", "Home", check=False)
    time.sleep(0.08)
    xdo("key", "shift+End", check=False)
    time.sleep(0.12)
    delay_ms = str(int(step.get("type_delay_ms", 55)))
    xdo("type", "--delay", delay_ms, replacement, check=False)
    time.sleep(float(step.get("pause_after_edit", 0.4)))
    hide_mouse()


def uncomment_and_set_true_current_line(window_id: str, step: dict[str, Any]) -> None:
    focus_editor(window_id)
    time.sleep(float(step.get("pause_before_edit", 0.5)))
    xdo("key", "Home", check=False)
    time.sleep(0.08)
    for _ in range(int(step.get("delete_prefix_chars", 2))):
        xdo("key", "Delete", check=False)
        time.sleep(0.1)
    time.sleep(float(step.get("pause_after_uncomment", 0.25)))
    xdo("key", "End", check=False)
    time.sleep(0.08)
    xdo("key", "ctrl+shift+Left", check=False)
    time.sleep(0.12)
    delay_ms = str(int(step.get("type_delay_ms", 55)))
    xdo("type", "--delay", delay_ms, "true", check=False)
    time.sleep(float(step.get("pause_after_edit", 0.4)))
    hide_mouse()


def discard_recording_edits(window_id: str) -> None:
    xdo("windowraise", window_id, check=False)
    xdo("windowfocus", window_id, check=False)
    for key in ("Escape", "Escape"):
        xdo("key", key, check=False)
        time.sleep(0.12)
    for step in edited_runbook_steps():
        window_id = open_file_at_line(str(step["file"]), int(step.get("line", 1)))
        focus_editor(window_id)
        xdo("key", "ctrl+alt+shift+r", check=False)
        time.sleep(0.35)
    close_all_editors(window_id)


def write_state(window_id: str) -> None:
    STATE_FILE.parent.mkdir(parents=True, exist_ok=True)
    STATE_FILE.write_text(json.dumps({"window_id": window_id, "updated_at": time.time()}, indent=2))


def close_window() -> None:
    if not STATE_FILE.is_file():
        terminate_recording_code()
        return
    try:
        state = json.loads(STATE_FILE.read_text())
    except json.JSONDecodeError:
        state = {}
    window_id = str(state.get("window_id", ""))
    if window_id:
        try:
            discard_recording_edits(window_id)
        except Exception:
            pass
        xdo("windowclose", window_id, check=False)
        time.sleep(0.5)
    terminate_recording_code()
    STATE_FILE.unlink(missing_ok=True)


def recording_code_pids() -> list[int]:
    profile = "/home/ops/dev/vnc/vscode-recording/profile"
    pattern = rf"/home/ops/dev/vnc/vscode-desktop/code-deb-root/usr/share/code/code .*--user-data-dir={profile}"
    result = subprocess.run(["pgrep", "-f", pattern], text=True, capture_output=True, check=False)
    pids: list[int] = []
    current = os.getpid()
    for raw in result.stdout.splitlines():
        try:
            pid = int(raw.strip())
        except ValueError:
            continue
        if pid != current:
            pids.append(pid)
    return pids


def terminate_recording_code() -> None:
    pids = recording_code_pids()
    for pid in pids:
        try:
            os.kill(pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    deadline = time.time() + 3
    while time.time() < deadline:
        if not recording_code_pids():
            return
        time.sleep(0.1)
    for pid in recording_code_pids():
        try:
            os.kill(pid, signal.SIGKILL)
        except ProcessLookupError:
            pass


def mark_ready() -> None:
    ready = os.environ.get("DEMO_VSCODE_READY_FILE")
    if ready:
        Path(ready).parent.mkdir(parents=True, exist_ok=True)
        Path(ready).write_text("ready\n")


def run_scene(scene_id: str) -> None:
    scene = scene_by_id(scene_id)
    human = scene.get("human") or {}
    steps = human.get("steps") or []
    if not steps:
        raise RuntimeError(f"{scene_id} has no VS Code steps")
    close_window()
    window_id = open_workspace()
    window_id = preload_tabs(window_id, steps)
    mark_ready()
    time.sleep(float(human.get("initial_pause", 1.0)))
    previous_file: str | None = None
    for index, step in enumerate(steps):
        window_id = show_file(window_id, step)
        previous_file = str(step["file"])
        pause = float(step.get("pause_after", human.get("pause_after", 3.0)))
        time.sleep(max(0.0, pause))
        move_window(window_id)
        if index + 1 < len(steps) and str(steps[index + 1]["file"]) != previous_file:
            focus_explorer(window_id)
    time.sleep(float(human.get("scene_end_pause", 0.8)))
    hide_mouse()


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser()
    parser.add_argument("scene", nargs="?")
    parser.add_argument("--close", action="store_true")
    args = parser.parse_args()
    if args.close:
        close_window()
        return 0
    if not args.scene:
        print("usage: vscode_scene_driver.py <scene-id> [--close]", file=sys.stderr)
        return 2
    run_scene(args.scene)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
