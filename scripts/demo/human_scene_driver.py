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
TERMINAL_STATE = ROOT / "demo-artifacts/pids/human-terminal.json"
DEFAULT_TERMINAL_TITLE = "Carbon ClickHouse Sink Tutorial"


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


def runbook() -> dict[str, Any]:
    return yaml.safe_load((ROOT / "scripts/demo/runbook.yaml").read_text())


def scene_by_id(scene_id: str) -> dict[str, Any]:
    for scene in runbook()["scenes"]:
        if scene["id"] == scene_id:
            return scene
    raise KeyError(scene_id)


def run(cmd: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, env=os.environ.copy(), text=True, capture_output=True, check=check)


def wait_for_window(title: str) -> str:
    for _ in range(80):
        result = run(["xdotool", "search", "--name", title], check=False)
        ids = [line.strip() for line in result.stdout.splitlines() if line.strip()]
        if ids:
            return ids[-1]
        time.sleep(0.15)
    raise RuntimeError(f"terminal window not found: {title}")


def xdo(*args: str) -> None:
    subprocess.run(["xdotool", *args], cwd=ROOT, env=os.environ.copy(), check=True)


def center_window(window_id: str) -> None:
    width = int(os.environ.get("DEMO_TERMINAL_WIDTH", "1800"))
    height = int(os.environ.get("DEMO_TERMINAL_HEIGHT", "860"))
    screen_w, screen_h = [int(part) for part in os.environ.get("DEMO_SCREEN_SIZE", "1920x1080").split("x", 1)]
    x = max(0, (screen_w - width) // 2)
    y = max(0, (screen_h - height) // 2)
    subprocess.run(["wmctrl", "-ir", window_id, "-e", f"0,{x},{y},{width},{height}"], check=True)
    time.sleep(0.4)
    xdo("mousemove", str(x + 120), str(y + 70))
    # Xvfb normally has no EWMH-compliant window manager, so windowactivate can
    # fail even when the xterm is visible. Focusing by id keeps this deterministic.
    xdo("windowfocus", window_id)
    xdo("click", "1")
    hide_mouse()


def type_command(command: str, delay_ms: int, *, pre_enter_pause: float | None = None) -> None:
    xdo("type", "--clearmodifiers", "--delay", str(delay_ms), "--", command)
    if pre_enter_pause is None:
        pre_enter_pause = float(os.environ.get("DEMO_PRE_ENTER_PAUSE_SECONDS", "0.55"))
    time.sleep(max(0.0, pre_enter_pause))
    xdo("key", "Return")


def hide_mouse() -> None:
    screen_w, screen_h = [int(part) for part in os.environ.get("DEMO_SCREEN_SIZE", "1920x1080").split("x", 1)]
    xdo("mousemove", str(screen_w - 2), str(screen_h - 2))


def env_flag(name: str, default: bool = False) -> bool:
    value = os.environ.get(name)
    if value is None:
        return default
    return value.strip().lower() in {"1", "true", "yes", "on"}


def pid_alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except OSError:
        return False
    return True


def window_alive(window_id: str) -> bool:
    return run(["xdotool", "getwindowname", window_id], check=False).returncode == 0


def read_terminal_state() -> dict[str, Any] | None:
    if not TERMINAL_STATE.is_file():
        return None
    try:
        return json.loads(TERMINAL_STATE.read_text())
    except json.JSONDecodeError:
        TERMINAL_STATE.unlink(missing_ok=True)
        return None


def write_terminal_state(pid: int, window_id: str, title: str) -> None:
    TERMINAL_STATE.parent.mkdir(parents=True, exist_ok=True)
    TERMINAL_STATE.write_text(
        json.dumps(
            {
                "pid": pid,
                "window_id": window_id,
                "title": title,
                "created_at": time.time(),
            },
            indent=2,
        )
    )


def close_reusable_terminal() -> None:
    state = read_terminal_state()
    TERMINAL_STATE.unlink(missing_ok=True)
    if not state:
        return
    try:
        pid = int(state.get("pid"))
    except (TypeError, ValueError):
        return
    if not pid_alive(pid):
        return
    os.kill(pid, signal.SIGTERM)
    for _ in range(30):
        if not pid_alive(pid):
            return
        time.sleep(0.1)
    if pid_alive(pid):
        os.kill(pid, signal.SIGKILL)


def existing_terminal(title: str) -> tuple[int, str] | None:
    state = read_terminal_state()
    if not state:
        return None
    try:
        pid = int(state.get("pid"))
        window_id = str(state.get("window_id"))
    except (TypeError, ValueError):
        TERMINAL_STATE.unlink(missing_ok=True)
        return None
    if state.get("title") != title or not pid_alive(pid) or not window_alive(window_id):
        TERMINAL_STATE.unlink(missing_ok=True)
        return None
    return pid, window_id


def launch_terminal(title: str, env: dict[str, str]) -> tuple[subprocess.Popen[bytes], str]:
    proc = subprocess.Popen(
        [
            "xterm",
            "-fa",
            "Monospace",
            "-fs",
            os.environ.get("DEMO_TERMINAL_FONT_SIZE", "10"),
            "-geometry",
            os.environ.get("DEMO_TERMINAL_GEOMETRY", "250x56"),
            "-bg",
            os.environ.get("DEMO_TERMINAL_BG", "#1e1e2e"),
            "-fg",
            os.environ.get("DEMO_TERMINAL_FG", "#cdd6f4"),
            "-cr",
            os.environ.get("DEMO_TERMINAL_CURSOR", "#f5e0dc"),
            "-xrm",
            "XTerm*color0: #45475a",
            "-xrm",
            "XTerm*color1: #f38ba8",
            "-xrm",
            "XTerm*color2: #a6e3a1",
            "-xrm",
            "XTerm*color3: #f9e2af",
            "-xrm",
            "XTerm*color4: #89b4fa",
            "-xrm",
            "XTerm*color5: #cba6f7",
            "-xrm",
            "XTerm*color6: #94e2d5",
            "-xrm",
            "XTerm*color7: #bac2de",
            "-xrm",
            "XTerm*color8: #585b70",
            "-xrm",
            "XTerm*color15: #a6adc8",
            "-title",
            title,
            "-e",
            "bash",
            "--noprofile",
            "--norc",
            "-i",
        ],
        cwd=ROOT,
        env=env,
    )
    window_id = wait_for_window(title)
    center_window(window_id)
    return proc, window_id


def run_terminal_scene(scene_id: str, *, dry_run: bool = False) -> int:
    scene = scene_by_id(scene_id)
    human = scene.get("human") or {}
    steps = human.get("steps") or []
    if not steps:
        raise RuntimeError(f"scene has no human terminal steps: {scene_id}")

    reuse_terminal = env_flag("DEMO_REUSE_TERMINAL", False)
    title = os.environ.get("DEMO_TERMINAL_TITLE", DEFAULT_TERMINAL_TITLE if reuse_terminal else f"{DEFAULT_TERMINAL_TITLE} - {scene_id}")
    env = os.environ.copy()
    env.setdefault("DISPLAY", os.environ.get("DEMO_DISPLAY", ":95"))
    env["PS1"] = "carbon-demo$ "
    env["VIRTUAL_ENV_DISABLE_PROMPT"] = "1"
    env.setdefault("DATABASE_URL", "http://carbon:carbon@localhost:8123")
    if env.get("SOLANA_RPC_URL") and not env.get("RPC_URL"):
        env["RPC_URL"] = env["SOLANA_RPC_URL"]
    if env.get("JUPITER_START_SLOT") and not env.get("BLOCK_CRAWLER_START_SLOT"):
        env["BLOCK_CRAWLER_START_SLOT"] = env["JUPITER_START_SLOT"]
    if env.get("JUPITER_END_SLOT") and not env.get("BLOCK_CRAWLER_END_SLOT"):
        env["BLOCK_CRAWLER_END_SLOT"] = env["JUPITER_END_SLOT"]
    env.setdefault("BLOCK_CRAWLER_HEAD_LAG_SLOTS", "3")
    env.setdefault("LOG_LEVEL", "info")

    if dry_run:
        for step in steps:
            print(step["command"])
        return 0

    proc: subprocess.Popen[bytes] | None = None
    try:
        terminal = existing_terminal(title) if reuse_terminal else None
        new_terminal = terminal is None
        if terminal:
            _, window_id = terminal
        else:
            proc, window_id = launch_terminal(title, env)
            if reuse_terminal:
                write_terminal_state(proc.pid, window_id, title)
        center_window(window_id)
        if new_terminal:
            time.sleep(0.8)
        elif reuse_terminal:
            xdo("key", "Return")
            time.sleep(0.4)
        for step in steps:
            command = str(step["command"])
            delay = int(step.get("type_delay_ms", os.environ.get("DEMO_TYPE_DELAY_MS", "38")))
            pre_enter_pause = float(step.get("pre_enter_pause", os.environ.get("DEMO_PRE_ENTER_PAUSE_SECONDS", "0.55")))
            pause = float(step.get("pause_after", 2.0))
            type_command(command, delay, pre_enter_pause=pre_enter_pause)
            hide_mouse()
            time.sleep(max(0.2, pause))
        scene_end_pause = float(human.get("scene_end_pause", os.environ.get("DEMO_SCENE_END_PAUSE_SECONDS", "2.0")))
        time.sleep(max(0.0, scene_end_pause))
        if not reuse_terminal and proc is not None:
            type_command("exit", 2, pre_enter_pause=0.15)
            try:
                proc.wait(timeout=8)
            except subprocess.TimeoutExpired:
                proc.terminate()
                proc.wait(timeout=4)
    finally:
        if proc is not None and proc.poll() is None and not reuse_terminal:
            proc.terminate()
    return 0


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser()
    parser.add_argument("scene", nargs="?")
    parser.add_argument("--close-terminal", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.close_terminal:
        close_reusable_terminal()
        return 0
    if not args.scene:
        parser.error("scene is required unless --close-terminal is used")
    return run_terminal_scene(args.scene, dry_run=args.dry_run)


if __name__ == "__main__":
    raise SystemExit(main())
