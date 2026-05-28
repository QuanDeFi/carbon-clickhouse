#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path
from urllib.parse import quote


ROOT = Path(__file__).resolve().parents[2]


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


def run(cmd: list[str], *, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, env=os.environ.copy(), text=True, capture_output=True, check=check)


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
    path = result.stdout.strip()
    if not Path(path).is_file():
        raise RuntimeError("could not resolve a Chromium executable")
    return path


def wait_for_chrome_window() -> str:
    for _ in range(120):
        result = run(["xdotool", "search", "--onlyvisible", "--class", "chrome"], check=False)
        ids = [line.strip() for line in result.stdout.splitlines() if line.strip().isdigit()]
        if ids:
            return ids[-1]
        time.sleep(0.25)
    raise RuntimeError("visible Chromium window was not found on the demo display")


def screen_size() -> tuple[int, int]:
    return tuple(int(part) for part in os.environ.get("DEMO_SCREEN_SIZE", "1920x1080").split("x", 1))  # type: ignore[return-value]


def single_frame() -> tuple[int, int, int, int]:
    width, height = screen_size()
    margin = int(os.environ.get("DEMO_WINDOW_MARGIN", "60"))
    frame_w = int(os.environ.get("DEMO_WINDOW_WIDTH", str(width - margin * 2)))
    frame_h = int(os.environ.get("DEMO_WINDOW_HEIGHT", str(height - margin * 2)))
    return max(0, (width - frame_w) // 2), max(0, (height - frame_h) // 2), min(frame_w, width), min(frame_h, height)


def window_shape(window_id: str) -> None:
    x, y, width, height = single_frame()
    subprocess.run(["xdotool", "windowmove", window_id, str(x), str(y)], cwd=ROOT, env=os.environ.copy(), check=False)
    subprocess.run(["xdotool", "windowsize", window_id, str(width), str(height)], cwd=ROOT, env=os.environ.copy(), check=False)
    subprocess.run(["xdotool", "windowfocus", window_id], cwd=ROOT, env=os.environ.copy(), check=False)
    subprocess.run(["xdotool", "mousemove", str(width - 2), str(height - 2)], cwd=ROOT, env=os.environ.copy(), check=False)
    time.sleep(1.2)


def hide_mouse() -> None:
    width, height = [int(part) for part in os.environ.get("DEMO_SCREEN_SIZE", "1920x1080").split("x", 1)]
    subprocess.run(["xdotool", "mousemove", str(width - 2), str(height - 2)], cwd=ROOT, env=os.environ.copy(), check=False)


def capture_screenshot(scene_id: str) -> Path:
    screenshot_dir = ROOT / "demo-artifacts/screenshots"
    screenshot_dir.mkdir(parents=True, exist_ok=True)
    output = screenshot_dir / f"{scene_id}.png"
    size = os.environ.get("DEMO_SCREEN_SIZE", "1920x1080")
    display = os.environ.get("DISPLAY", os.environ.get("DEMO_DISPLAY", ":95"))
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
            size,
            "-i",
            display,
            "-frames:v",
            "1",
            str(output),
        ],
        cwd=ROOT,
        env=os.environ.copy(),
        check=True,
    )
    return output


def launch(url: str, scene_id: str, *, force_dark: bool = True, offscreen: bool = False) -> subprocess.Popen:
    profile = ROOT / "demo-artifacts/browser-profiles" / scene_id
    if profile.exists():
        shutil.rmtree(profile)
    profile.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env["DISPLAY"] = env.get("DISPLAY", env.get("DEMO_DISPLAY", ":95"))
    x, y, width, height = single_frame()
    # Launch off-screen when the caller will reveal the window itself, so the
    # page's first (white) paint and the still-visible terminal behind it are
    # never recorded; window_shape() then moves it on-screen once painted.
    pos_x, pos_y = (-32000, -32000) if offscreen else (x, y)
    args = [
        chrome_executable(),
        "--no-first-run",
        "--no-default-browser-check",
        "--no-sandbox",
        "--disable-gpu",
        "--disable-dev-shm-usage",
        "--disable-extensions",
        "--disable-infobars",
        "--test-type",
    ]
    # Chrome's force-dark filter briefly inverts already-dark content while a
    # page first paints, which shows as a white flash. Slides are authored
    # dark, so they must not be force-darkened; the Prometheus graph UI is
    # light and still needs it.
    if force_dark:
        args += ["--force-dark-mode", "--enable-features=WebUIDarkMode"]
    args += [
        f"--window-position={pos_x},{pos_y}",
        f"--window-size={width},{height}",
        f"--user-data-dir={profile}",
        f"--app={url}",
    ]
    return subprocess.Popen(
        args,
        cwd=ROOT,
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )


def run_scene(scene_id: str, url: str, seconds: float, *, force_dark: bool = True, offscreen: bool = False) -> int:
    proc = launch(url, scene_id, force_dark=force_dark, offscreen=offscreen)
    try:
        window_id = wait_for_chrome_window()
        window_shape(window_id)
        hide_mouse()
        # Once the page is painted and on-screen, signal readiness so the
        # orchestrator can start recording (it covers the terminal, so the
        # terminal is never recorded before the slide appears).
        ready_file = os.environ.get("DEMO_SLIDE_READY_FILE")
        if ready_file:
            Path(ready_file).write_text(f"{time.time()}\n")
        capture_screenshot(scene_id)
        # In delayed-record mode the recorder starts after DEMO_SLIDE_READY_FILE
        # is written, so the requested duration must be counted from readiness,
        # not from browser launch. Non-delayed callers already record launch
        # and shaping time, so keep their historical shorter hold.
        if ready_file:
            time.sleep(max(0.0, seconds))
        else:
            time.sleep(max(0.0, seconds - 1.5))
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
    return 0


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="mode", required=True)
    slide = sub.add_parser("slide")
    slide.add_argument("scene_id")
    slide.add_argument("html")
    slide.add_argument("--seconds", type=float, default=10.0)
    prom = sub.add_parser("prometheus")
    prom.add_argument("scene_id")
    prom.add_argument("query")
    prom.add_argument("--seconds", type=float, default=28.0)
    args = parser.parse_args()

    os.environ.setdefault("DEMO_DISPLAY", ":95")
    os.environ.setdefault("DISPLAY", os.environ["DEMO_DISPLAY"])

    if args.mode == "slide":
        path = Path(args.html)
        if not path.is_absolute():
            path = ROOT / path
        # Slides are authored dark; force-dark would flash white on first paint.
        # Prepare off-screen and reveal once painted so neither the white first
        # paint nor the terminal behind it is recorded.
        return run_scene(args.scene_id, path.resolve().as_uri(), args.seconds, force_dark=False, offscreen=True)

    url = f"http://localhost:9090/graph?g0.expr={quote(args.query)}&g0.tab=1&g0.show_tree=0"
    return run_scene(args.scene_id, url, args.seconds, force_dark=True)


if __name__ == "__main__":
    raise SystemExit(main())
