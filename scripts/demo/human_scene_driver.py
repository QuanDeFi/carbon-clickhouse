#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import os
import random
import signal
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parents[2]
TERMINAL_STATE = ROOT / "demo-artifacts/pids/human-terminal.json"
TERMINAL_READY_FILE = ROOT / "demo-artifacts/pids/human-terminal-ready"
DEFAULT_TERMINAL_TITLE = "Carbon ClickHouse Sink Tutorial"


def terminal_state_file(name: str) -> Path:
    safe = "".join(char if char.isalnum() or char in {"-", "_"} else "-" for char in name.strip()) or "default"
    return ROOT / "demo-artifacts/pids" / f"human-terminal-{safe}.json"


def terminal_ready_file(name: str) -> Path:
    safe = "".join(char if char.isalnum() or char in {"-", "_"} else "-" for char in name.strip()) or "default"
    return ROOT / "demo-artifacts/pids" / f"human-terminal-ready-{safe}"


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
        ids = [line.strip() for line in result.stdout.splitlines() if line.strip().isdigit()]
        if ids:
            return ids[-1]
        time.sleep(0.15)
    raise RuntimeError(f"terminal window not found: {title}")


def demo_windows() -> list[tuple[str, str]]:
    result = run(["wmctrl", "-l"], check=False)
    windows: list[tuple[str, str]] = []
    for line in result.stdout.splitlines():
        parts = line.split(None, 3)
        if len(parts) < 4:
            continue
        window_id, title = parts[0], parts[3]
        if "Carbon ClickHouse" in title or "ClickHouse" in title or "Grafana" in title:
            windows.append((window_id, title))
    return windows


def xdo(*args: str) -> None:
    subprocess.run(["xdotool", *args], cwd=ROOT, env=os.environ.copy(), check=True)


def screen_size() -> tuple[int, int]:
    return tuple(int(part) for part in os.environ.get("DEMO_SCREEN_SIZE", "1920x1080").split("x", 1))  # type: ignore[return-value]


def single_frame() -> tuple[int, int, int, int]:
    screen_w, screen_h = screen_size()
    margin = int(os.environ.get("DEMO_WINDOW_MARGIN", os.environ.get("DEMO_TERMINAL_MARGIN", "60")))
    width = int(os.environ.get("DEMO_WINDOW_WIDTH", str(screen_w - (margin * 2))))
    height = int(os.environ.get("DEMO_WINDOW_HEIGHT", str(screen_h - (margin * 2))))
    return max(0, (screen_w - width) // 2), max(0, (screen_h - height) // 2), min(width, screen_w), min(height, screen_h)


def overview_frames(count: int) -> list[tuple[int, int, int, int]]:
    screen_w, screen_h = screen_size()
    margin = int(os.environ.get("DEMO_OVERVIEW_MARGIN", "80"))
    gap = int(os.environ.get("DEMO_OVERVIEW_GAP", "42"))
    cols = 2 if count <= 4 else 3
    rows = max(1, (count + cols - 1) // cols)
    width = (screen_w - (margin * 2) - (gap * (cols - 1))) // cols
    height = (screen_h - (margin * 2) - (gap * (rows - 1))) // rows
    return [
        (margin + (index % cols) * (width + gap), margin + (index // cols) * (height + gap), width, height)
        for index in range(count)
    ]


def window_geometry(window_id: str) -> tuple[int, int, int, int] | None:
    result = run(["xdotool", "getwindowgeometry", "--shell", window_id], check=False)
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


def move_window(window_id: str, frame: tuple[int, int, int, int]) -> None:
    x, y, width, height = frame
    subprocess.run(["wmctrl", "-ir", window_id, "-e", f"0,{x},{y},{width},{height}"], check=False)
    # In the tutorial Xvfb session there may be no EWMH-compliant window manager.
    # wmctrl can then report success without resizing xterm, so enforce the
    # pixel frame directly through xdotool as well.
    subprocess.run(["xdotool", "windowmove", window_id, str(x), str(y)], cwd=ROOT, env=os.environ.copy(), check=False)
    subprocess.run(["xdotool", "windowsize", window_id, str(width), str(height)], cwd=ROOT, env=os.environ.copy(), check=False)


def animate_window(window_id: str, target: tuple[int, int, int, int], *, steps: int = 8) -> None:
    start = window_geometry(window_id)
    if start is None:
        move_window(window_id, target)
        time.sleep(0.2)
        return
    for step in range(1, steps + 1):
        t = step / steps
        eased = 1 - (1 - t) * (1 - t)
        frame = tuple(int(start[i] + (target[i] - start[i]) * eased) for i in range(4))
        move_window(window_id, frame)  # type: ignore[arg-type]
        time.sleep(0.035)


def show_window_overview(target_window_id: str) -> None:
    if not env_flag("DEMO_WINDOW_OVERVIEW", True):
        return
    label = run(["xdotool", "getwindowname", target_window_id], check=False).stdout.strip() or "terminal"
    run([sys.executable, "scripts/demo/window_transition.py", label], check=False)


def read_ready_token() -> str | None:
    try:
        return TERMINAL_READY_FILE.read_text().strip()
    except FileNotFoundError:
        return None


def wait_for_prompt_change(previous: str | None, timeout: float, *, allow_same: bool = False) -> str:
    deadline = time.time() + timeout
    last_seen = previous
    while time.time() < deadline:
        current = read_ready_token()
        if current and (allow_same or current != previous):
            return current
        if current:
            last_seen = current
        time.sleep(0.15)
    detail = f"last prompt marker: {last_seen or '<missing>'}"
    raise TimeoutError(f"terminal prompt did not return within {timeout:.1f}s ({detail})")


def center_window(window_id: str, *, transition: bool = True) -> None:
    if "DEMO_TERMINAL_WIDTH" not in os.environ and "DEMO_TERMINAL_HEIGHT" not in os.environ:
        x, y, width, height = single_frame()
    else:
        screen_w, screen_h = screen_size()
        width = int(os.environ.get("DEMO_TERMINAL_WIDTH", str(single_frame()[2])))
        height = int(os.environ.get("DEMO_TERMINAL_HEIGHT", str(single_frame()[3])))
        x = int(os.environ.get("DEMO_TERMINAL_X", str(max(0, (screen_w - width) // 2))))
        y = int(os.environ.get("DEMO_TERMINAL_Y", str(max(0, (screen_h - height) // 2))))
    move_window(window_id, (x, y, width, height))
    time.sleep(0.4)
    xdo("mousemove", str(x + 120), str(y + 70))
    # Xvfb normally has no EWMH-compliant window manager, so windowactivate can
    # fail even when the xterm is visible. Focusing by id keeps this deterministic.
    xdo("windowfocus", window_id)
    xdo("click", "1")
    hide_mouse()
    if transition and not env_flag("DEMO_SUPPRESS_DRIVER_TRANSITION", False):
        show_window_overview(window_id)
        xdo("windowfocus", window_id)
        xdo("click", "1")
        hide_mouse()


class TypingProfile:
    """Deterministic human-ish typing model for visible tutorial commands."""

    def __init__(self, *, base_delay_ms: float, seed: str, text_length: int = 0) -> None:
        self.base_delay_ms = base_delay_ms
        self.rng = random.Random(seed)
        self.min_delay_ms = float(os.environ.get("DEMO_TYPE_MIN_DELAY_MS", "28"))
        self.max_delay_ms = float(os.environ.get("DEMO_TYPE_MAX_DELAY_MS", "145"))
        self.sigma = float(os.environ.get("DEMO_TYPE_LOGNORMAL_SIGMA", "0.34"))
        if text_length <= 30:
            self.length_factor = 1.0
        else:
            self.length_factor = max(0.67, 1.0 - ((min(text_length, 110) - 30) / 80.0) * 0.33)
        self.space_pause = self._range("DEMO_TYPE_SPACE_PAUSE_MS", 45, 125)
        self.burst_pause = self._range("DEMO_TYPE_BURST_PAUSE_MS", 45, 160)
        self.operator_pause = self._range("DEMO_TYPE_OPERATOR_PAUSE_MS", 130, 340)
        self.burst_len = self._int_range("DEMO_TYPE_BURST_CHARS", 8, 18)
        self.pre_enter = self._range("DEMO_PRE_ENTER_PAUSE_MS", 650, 1000)

    @staticmethod
    def _range(name: str, default_low: int, default_high: int) -> tuple[float, float]:
        raw = os.environ.get(name)
        if not raw:
            return float(default_low), float(default_high)
        left, right = raw.replace(":", ",").split(",", 1)
        return float(left.strip()), float(right.strip())

    @staticmethod
    def _int_range(name: str, default_low: int, default_high: int) -> tuple[int, int]:
        low, high = TypingProfile._range(name, default_low, default_high)
        return int(low), int(high)

    def uniform_ms(self, bounds: tuple[float, float]) -> float:
        low, high = bounds
        return self.rng.uniform(low, high)

    def next_burst_length(self) -> int:
        low, high = self.burst_len
        return self.rng.randint(low, high)

    def char_delay_ms(self, char: str, previous: str | None) -> float:
        # Human keystroke timing is positively skewed, not fixed or Gaussian.
        delay = self.rng.lognormvariate(0.0, self.sigma) * self.base_delay_ms
        if char in "'\"`$(){}[]<>|&;:=+":
            delay *= 1.45
        elif char in "-_./\\":
            delay *= 1.20
        elif char.isupper():
            delay *= 1.12
        if previous and previous == char:
            delay *= 1.18
        delay *= self.length_factor
        min_delay = self.min_delay_ms * self.length_factor
        max_delay = self.max_delay_ms * max(self.length_factor, 0.82)
        return max(min_delay, min(max_delay, delay))

    def boundary_pause_ms(self, char: str, next_char: str | None) -> float:
        if char == " ":
            return self.uniform_ms(self.space_pause) * self.length_factor
        if char in "|&;":
            return self.uniform_ms(self.operator_pause) * self.length_factor
        if char == "\\" or next_char in {"|", "&", ";"}:
            return self.uniform_ms(self.operator_pause) * 0.65 * self.length_factor
        return 0.0


def typing_seed(command: str, scene_id: str, step_index: int) -> str:
    configured = os.environ.get("DEMO_TYPING_SEED", "carbon-clickhouse-tutorial")
    digest = hashlib.sha256(f"{configured}:{scene_id}:{step_index}:{command}".encode()).hexdigest()
    return digest[:16]


def type_text_human(command: str, profile: TypingProfile) -> None:
    burst_remaining = profile.next_burst_length()
    previous: str | None = None
    for index, char in enumerate(command):
        xdo("type", "--clearmodifiers", "--delay", "0", "--", char)
        next_char = command[index + 1] if index + 1 < len(command) else None
        time.sleep(profile.char_delay_ms(char, previous) / 1000.0)
        boundary_pause = profile.boundary_pause_ms(char, next_char)
        if boundary_pause > 0:
            time.sleep(boundary_pause / 1000.0)
        burst_remaining -= 1
        if burst_remaining <= 0 and next_char and char != " ":
            time.sleep(profile.uniform_ms(profile.burst_pause) / 1000.0)
            burst_remaining = profile.next_burst_length()
        previous = char


def type_command(command: str, delay_ms: int, *, pre_enter_pause: float | None = None, scene_id: str = "scene", step_index: int = 0) -> None:
    if env_flag("DEMO_LEGACY_FIXED_TYPING", False):
        xdo("type", "--clearmodifiers", "--delay", str(delay_ms), "--", command)
    else:
        base_delay_ms = float(os.environ.get("DEMO_TYPE_BASE_DELAY_MS", str(delay_ms)))
        profile = TypingProfile(base_delay_ms=base_delay_ms, seed=typing_seed(command, scene_id, step_index), text_length=len(command))
        type_text_human(command, profile)
        if pre_enter_pause is None:
            pre_enter_pause = profile.uniform_ms(profile.pre_enter) / 1000.0
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
    global TERMINAL_STATE
    state_files = {TERMINAL_STATE}
    state_files.update((ROOT / "demo-artifacts/pids").glob("human-terminal-*.json"))
    for state_file in state_files:
        TERMINAL_STATE = state_file
        state = read_terminal_state()
        state_file.unlink(missing_ok=True)
        if not state:
            continue
        try:
            pid = int(state.get("pid"))
        except (TypeError, ValueError):
            continue
        if not pid_alive(pid):
            continue
        os.kill(pid, signal.SIGTERM)
        for _ in range(30):
            if not pid_alive(pid):
                break
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
    TERMINAL_READY_FILE.parent.mkdir(parents=True, exist_ok=True)
    TERMINAL_READY_FILE.unlink(missing_ok=True)
    proc = subprocess.Popen(
        [
            "xterm",
            "-fa",
            "Monospace",
            "-fs",
            os.environ.get("DEMO_TERMINAL_FONT_SIZE", "12"),
            "-geometry",
            os.environ.get("DEMO_TERMINAL_GEOMETRY", "210x48"),
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
    center_window(window_id, transition=False)
    wait_for_prompt_change(None, 8.0)
    return proc, window_id


def run_terminal_scene(scene_id: str, *, dry_run: bool = False, prepare_only: bool = False) -> int:
    global TERMINAL_STATE, TERMINAL_READY_FILE
    scene = scene_by_id(scene_id)
    human = scene.get("human") or {}
    steps = human.get("steps") or []
    if not steps:
        raise RuntimeError(f"scene has no human terminal steps: {scene_id}")

    terminal_name = str(human.get("terminal_name") or ("main" if env_flag("DEMO_REUSE_TERMINAL", False) else scene_id))
    TERMINAL_STATE = terminal_state_file(terminal_name)
    TERMINAL_READY_FILE = terminal_ready_file(terminal_name)

    reuse_terminal = bool(human.get("reuse_terminal", env_flag("DEMO_REUSE_TERMINAL", False)))
    keep_open = bool(human.get("keep_open", False))
    title = os.environ.get("DEMO_TERMINAL_TITLE", str(human.get("terminal_title") or (DEFAULT_TERMINAL_TITLE if reuse_terminal else f"{DEFAULT_TERMINAL_TITLE} - {scene_id}")))
    env = os.environ.copy()
    env.setdefault("DISPLAY", os.environ.get("DEMO_DISPLAY", ":95"))
    for key, env_key in {
        "x": "DEMO_TERMINAL_X",
        "y": "DEMO_TERMINAL_Y",
        "width": "DEMO_TERMINAL_WIDTH",
        "height": "DEMO_TERMINAL_HEIGHT",
        "font_size": "DEMO_TERMINAL_FONT_SIZE",
        "geometry": "DEMO_TERMINAL_GEOMETRY",
    }.items():
        value = human.get("window", {}).get(key) if isinstance(human.get("window"), dict) else human.get(key)
        if value is not None:
            env[env_key] = str(value)
            os.environ[env_key] = str(value)
    env["PS1"] = "carbon-demo$ "
    env["PROMPT_COMMAND"] = (
        "printf '%s %s\\n' \"$(date +%s%N)\" \"$?\" > "
        + "'" + str(TERMINAL_READY_FILE) + "'"
    )
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
            if reuse_terminal or keep_open:
                write_terminal_state(proc.pid, window_id, title)
        center_window(window_id, transition=not prepare_only)
        if prepare_only:
            hide_mouse()
            return 0
        if new_terminal:
            time.sleep(0.8)
        elif reuse_terminal:
            before = read_ready_token()
            xdo("key", "Return")
            wait_for_prompt_change(before, 8.0)
            time.sleep(0.4)
        for step_index, step in enumerate(steps):
            command = str(step["command"])
            delay = int(step.get("type_delay_ms", os.environ.get("DEMO_TYPE_BASE_DELAY_MS", os.environ.get("DEMO_TYPE_DELAY_MS", "50"))))
            pre_enter_pause = step.get("pre_enter_pause")
            if pre_enter_pause is not None:
                pre_enter_pause = float(pre_enter_pause)
            pause = float(step.get("pause_after", 2.0))
            prompt_before = read_ready_token()
            type_command(command, delay, pre_enter_pause=pre_enter_pause, scene_id=scene_id, step_index=step_index)
            if bool(step.get("wait_for_prompt", True)):
                wait_timeout = float(step.get("wait_for_prompt_timeout", os.environ.get("DEMO_WAIT_FOR_PROMPT_TIMEOUT_SECONDS", "180")))
                wait_for_prompt_change(prompt_before, wait_timeout)
            hide_mouse()
            time.sleep(max(0.2, pause))
        scene_end_pause = float(human.get("scene_end_pause", os.environ.get("DEMO_SCENE_END_PAUSE_SECONDS", "2.0")))
        time.sleep(max(0.0, scene_end_pause))
        if not reuse_terminal and not keep_open and proc is not None:
            type_command("exit", 2, pre_enter_pause=0.15, scene_id=scene_id, step_index=len(steps))
            try:
                proc.wait(timeout=8)
            except subprocess.TimeoutExpired:
                proc.terminate()
                proc.wait(timeout=4)
    finally:
        if proc is not None and proc.poll() is None and not reuse_terminal and not keep_open:
            proc.terminate()
    return 0


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser()
    parser.add_argument("scene", nargs="?")
    parser.add_argument("--close-terminal", action="store_true")
    parser.add_argument("--prepare-terminal", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    if args.close_terminal:
        close_reusable_terminal()
        return 0
    if not args.scene:
        parser.error("scene is required unless --close-terminal is used")
    return run_terminal_scene(args.scene, dry_run=args.dry_run, prepare_only=args.prepare_terminal)


if __name__ == "__main__":
    raise SystemExit(main())
