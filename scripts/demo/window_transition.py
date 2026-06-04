#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
STATE_DIR = ROOT / "demo-artifacts/window-transitions"
STATE_FILE = STATE_DIR / "state.json"


def screen_size() -> tuple[int, int]:
    raw = os.environ.get("DEMO_SCREEN_SIZE", "1920x1080")
    width, height = raw.split("x", 1)
    return int(width), int(height)


def window_frame() -> tuple[int, int, int, int]:
    width, height = screen_size()
    margin = int(os.environ.get("DEMO_WINDOW_MARGIN", "60"))
    return margin, margin, width - margin * 2, height - margin * 2


def chrome_executable() -> str:
    configured = os.environ.get("DEMO_CHROME_BIN")
    if configured and Path(configured).is_file():
        return configured
    for candidate in ("chromium", "chromium-browser", "google-chrome", "google-chrome-stable"):
        found = shutil.which(candidate)
        if found:
            return found
    result = subprocess.run(
        ["node", "-e", "const { chromium } = require('@playwright/test'); console.log(chromium.executablePath())"],
        cwd=ROOT / "scripts/demo/playwright",
        text=True,
        capture_output=True,
        check=True,
    )
    return result.stdout.strip()


def capture_display(path: Path) -> None:
    x, y, width, height = window_frame()
    display = os.environ.get("DISPLAY", os.environ.get("DEMO_DISPLAY", ":96"))
    path.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "x11grab",
            "-draw_mouse",
            "0",
            "-video_size",
            f"{width}x{height}",
            "-i",
            f"{display}+{x},{y}",
            "-frames:v",
            "1",
            str(path),
        ],
        cwd=ROOT,
        env=os.environ.copy(),
        check=True,
    )


def slot_for_label(label: str) -> int:
    normalized = label.lower()
    if any(token in normalized for token in ("token program", "token account", "token snapshot")):
        return 1
    if "grafana" in normalized or "metrics dashboard" in normalized:
        return 2
    if "clickstack" in normalized or "query telemetry" in normalized:
        return 3
    if "play" in normalized or "landing table" in normalized or "table inspection" in normalized:
        return 4
    if "async" in normalized:
        return 5
    return 0


def write_blackout_html(path: Path) -> None:
    width, height = screen_size()
    path.write_text(
        f"""<!doctype html>
<html>
<head>
<meta charset="utf-8">
<meta name="color-scheme" content="dark">
<style>
html, body {{
  margin: 0;
  width: {width}px;
  height: {height}px;
  overflow: hidden;
  background: #000;
  cursor: none;
}}
</style>
</head>
<body></body>
</html>
"""
    )


def find_transition_window(display: str, html_path: Path) -> str:
    fallback = ""
    deadline = time.time() + float(os.environ.get("DEMO_WINDOW_TRANSITION_START_TIMEOUT_SECONDS", "5.0"))
    while time.time() < deadline:
        found = subprocess.run(
            ["xdotool", "search", "--class", "chrom"],
            cwd=ROOT,
            env={**os.environ, "DISPLAY": display},
            text=True,
            capture_output=True,
            check=False,
        )
        if found.returncode == 0 and found.stdout.strip():
            ids = [line.strip() for line in found.stdout.splitlines() if line.strip().isdigit()]
            if ids:
                fallback = ids[-1]
            for candidate in reversed(ids):
                title = subprocess.run(
                    ["xdotool", "getwindowname", candidate],
                    cwd=ROOT,
                    env={**os.environ, "DISPLAY": display},
                    text=True,
                    capture_output=True,
                    check=False,
                )
                if "blackout.html" in title.stdout or html_path.name in title.stdout:
                    return candidate
        time.sleep(0.05)
    return fallback


def xdo(display: str, *args: str) -> None:
    subprocess.run(
        ["xdotool", *args],
        cwd=ROOT,
        env={**os.environ, "DISPLAY": display},
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )


def tutorial_windows(display: str) -> list[tuple[str, str]]:
    result = subprocess.run(
        ["wmctrl", "-l"],
        cwd=ROOT,
        env={**os.environ, "DISPLAY": display},
        text=True,
        capture_output=True,
        check=False,
    )
    windows: list[tuple[str, str]] = []
    for line in result.stdout.splitlines():
        parts = line.split(None, 3)
        if len(parts) < 4:
            continue
        raw_id, title = parts[0], parts[3]
        normalized = title.lower()
        if any(
            token in normalized
            for token in (
                "carbon clickhouse",
                "clickhouse",
                "clickstack",
                "grafana",
                "visual studio code",
                "chromium",
            )
        ):
            try:
                window_id = str(int(raw_id, 16))
            except ValueError:
                window_id = raw_id
            windows.append((window_id, title))
    return windows


def hide_other_tutorial_windows(display: str, target_window_id: str, transition_window_id: str = "") -> None:
    keep = {item for item in (target_window_id, transition_window_id) if item}
    for window_id, _title in tutorial_windows(display):
        if window_id in keep:
            continue
        xdo(display, "windowmove", window_id, "-32000", "-32000")


def place_target_behind_blackout(display: str, transition_window_id: str) -> None:
    target_window_id = os.environ.get("DEMO_WINDOW_TRANSITION_TARGET_WINDOW_ID", "").strip()
    if not target_window_id:
        return
    tx, ty, tw, th = window_frame()
    for args in (
        ("windowmove", target_window_id, str(tx), str(ty)),
        ("windowsize", target_window_id, str(tw), str(th)),
    ):
        xdo(display, *args)
    hide_other_tutorial_windows(display, target_window_id, transition_window_id)
    if transition_window_id:
        xdo(display, "windowraise", transition_window_id)
        xdo(display, "windowfocus", transition_window_id)


def mark_ready() -> None:
    ready_file = os.environ.get("DEMO_WINDOW_TRANSITION_READY_FILE")
    if not ready_file:
        return
    ready = Path(ready_file)
    ready.parent.mkdir(parents=True, exist_ok=True)
    ready.write_text(f"{time.time()}\n")


def run_transition(label: str) -> None:
    if os.environ.get("DEMO_WINDOW_OVERVIEW", "true").lower() in {"0", "false", "no", "off"}:
        mark_ready()
        return

    STATE_DIR.mkdir(parents=True, exist_ok=True)
    html_path = STATE_DIR / "blackout.html"
    write_blackout_html(html_path)

    width, height = screen_size()
    display = os.environ.get("DISPLAY", os.environ.get("DEMO_DISPLAY", ":96"))
    profile = STATE_DIR / f"profile-{int(time.time() * 1000)}"
    proc = subprocess.Popen(
        [
            chrome_executable(),
            "--no-first-run",
            "--no-default-browser-check",
            "--no-sandbox",
            "--disable-gpu",
            "--disable-dev-shm-usage",
            "--disable-extensions",
            "--disable-infobars",
            "--test-type",
            "--disable-backgrounding-occluded-windows",
            "--disable-renderer-backgrounding",
            "--disable-features=CalculateNativeWinOcclusion",
            "--ozone-platform=x11",
            "--window-position=0,0",
            f"--window-size={width},{height}",
            f"--user-data-dir={profile}",
            f"--app={html_path.resolve().as_uri()}",
        ],
        cwd=ROOT,
        env={**os.environ, "DISPLAY": display},
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    transition_seconds = float(os.environ.get("DEMO_WINDOW_TRANSITION_SECONDS", "0.75"))
    try:
        window_id = find_transition_window(display, html_path)
        time.sleep(float(os.environ.get("DEMO_WINDOW_TRANSITION_READY_DELAY_SECONDS", "0.18")))
        if window_id:
            xdo(display, "windowmove", window_id, "0", "0")
            xdo(display, "windowsize", window_id, str(width), str(height))
            xdo(display, "windowraise", window_id)
            xdo(display, "windowfocus", window_id)
        place_target_behind_blackout(display, window_id)
        mark_ready()
        time.sleep(max(0.1, transition_seconds))
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=4)
        except subprocess.TimeoutExpired:
            proc.kill()
        shutil.rmtree(profile, ignore_errors=True)

    STATE_FILE.write_text(json.dumps({"slot": slot_for_label(label), "label": label}, indent=2))


def capture_current_state(label: str) -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    slot = slot_for_label(label)
    target = STATE_DIR / f"slot-{slot}.png"
    capture_display(target)
    shutil.copyfile(target, STATE_DIR / "previous.png")
    STATE_FILE.write_text(json.dumps({"slot": slot, "label": label}, indent=2))


def reset() -> None:
    shutil.rmtree(STATE_DIR, ignore_errors=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("label", nargs="?")
    parser.add_argument("--reset", action="store_true")
    parser.add_argument(
        "--capture-current",
        metavar="LABEL",
        help="capture the current visible window region as the latest transition state for LABEL",
    )
    args = parser.parse_args()
    if args.reset:
        reset()
        return 0
    if args.capture_current:
        capture_current_state(args.capture_current)
        return 0
    if not args.label:
        parser.error("label is required unless --reset or --capture-current is used")
    run_transition(args.label)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
