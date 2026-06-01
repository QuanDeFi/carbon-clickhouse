#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parents[2]
STATE_FILE = ROOT / "demo-artifacts/pids/vscode-window.json"
DEFAULT_LAUNCHER = Path("/home/ops/dev/vnc/vscode-recording/launch-code-recording.sh")


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
    data.setdefault("DEMO_DISPLAY", ":95")
    data.setdefault("DISPLAY", data["DEMO_DISPLAY"])
    data.setdefault("DEMO_SCREEN_SIZE", "1920x1080")
    return data


def run(cmd: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, env=env(), text=True, capture_output=True, check=check)


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


def close_all_editors(window_id: str) -> None:
    xdo("windowraise", window_id, check=False)
    xdo("windowfocus", window_id, check=False)
    xdo("key", "ctrl+alt+shift+w", check=False)
    time.sleep(0.25)
    focus_explorer(window_id)


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


def launcher_path() -> Path:
    configured = os.environ.get("DEMO_VSCODE_LAUNCHER")
    launcher = Path(configured) if configured else DEFAULT_LAUNCHER
    if not launcher.is_file():
        raise FileNotFoundError(
            f"VS Code recording launcher not found: {launcher}. "
            "Run the desktop VS Code setup before recording VS Code scenes."
        )
    return launcher


def open_workspace() -> str:
    command = [str(launcher_path()), "--new-window", str(ROOT)]
    result = subprocess.run(command, cwd=ROOT, env=env(), text=True, capture_output=True)
    if result.returncode != 0:
        raise RuntimeError(result.stdout + result.stderr)
    window_id = wait_for_code_window(ROOT.name)
    move_window(window_id)
    focus_explorer(window_id)
    close_all_editors(window_id)
    write_state(window_id)
    return window_id


def open_file(step: dict[str, Any], *, first: bool) -> str:
    relative = str(step["file"])
    line = int(step.get("line", 1))
    path = ROOT / relative
    if not path.is_file():
        raise FileNotFoundError(path)
    if first:
        window_id = open_workspace()
        time.sleep(0.7)
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
    focus_explorer(window_id)
    write_state(window_id)
    print(f"opened {relative}:{line}")
    return window_id


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
    first = True
    for index, step in enumerate(steps):
        window_id = open_file(step, first=first)
        first = False
        if index == 0:
            # Let VS Code finish first paint before ffmpeg starts capturing.
            time.sleep(float(human.get("initial_pause", 1.0)))
            mark_ready()
        pause = float(step.get("pause_after", human.get("pause_after", 3.0)))
        time.sleep(max(0.0, pause))
        move_window(window_id)
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
