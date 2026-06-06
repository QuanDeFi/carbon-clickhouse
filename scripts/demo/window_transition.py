#!/usr/bin/env python3
from __future__ import annotations

import argparse
import html
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


def file_uri(path: Path) -> str:
    return html.escape(path.resolve().as_uri(), quote=True)


def write_blackout_html(path: Path, *, transition_seconds: float, target_image: Path | None = None, previous_image: Path | None = None) -> None:
    width, height = screen_size()
    x, y, frame_width, frame_height = window_frame()
    animation_seconds = max(0.1, transition_seconds - 0.12)
    if target_image and target_image.is_file() and previous_image and previous_image.is_file():
        image_markup = f"""
<div class="frame">
  <img class="previous" src="{file_uri(previous_image)}" alt="">
  <img class="target" src="{file_uri(target_image)}" alt="">
</div>
<script>
let started = false;
const start = () => {{
  if (started) return;
  started = true;
  document.body.classList.add("start");
}};
window.addEventListener("focus", () => setTimeout(start, 70), {{ once: true }});
const focusPoll = setInterval(() => {{
  if (!document.hasFocus()) return;
  clearInterval(focusPoll);
  setTimeout(start, 70);
}}, 30);
setTimeout(start, 2000);
</script>
"""
        frame_css = f"""
.frame {{
  position: absolute;
  left: {x}px;
  top: {y}px;
  width: {frame_width}px;
  height: {frame_height}px;
  overflow: hidden;
  background: #000;
}}
.frame img {{
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: fill;
}}
.previous {{
  opacity: 1;
}}
.target {{
  opacity: 0;
}}
body.start .previous {{
  animation: previous-fade {animation_seconds:.3f}s ease-in-out forwards;
}}
body.start .target {{
  animation: target-fade {animation_seconds:.3f}s ease-in-out forwards;
}}
@keyframes previous-fade {{
  0% {{ opacity: 1; }}
  25%, 100% {{ opacity: 0; }}
}}
@keyframes target-fade {{
  0%, 75% {{ opacity: 0; }}
  100% {{ opacity: 1; }}
}}
"""
    else:
        image_markup = ""
        frame_css = ""
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
{frame_css}
</style>
</head>
<body>{image_markup}</body>
</html>
"""
    )


def ensure_black_frame(path: Path) -> None:
    if path.is_file():
        return
    _, _, frame_width, frame_height = window_frame()
    subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            f"color=c=black:s={frame_width}x{frame_height}",
            "-frames:v",
            "1",
            str(path),
        ],
        cwd=ROOT,
        env=os.environ.copy(),
        check=True,
    )


def render_transition_video(
    path: Path,
    *,
    transition_seconds: float,
    target_image: Path | None = None,
    previous_image: Path | None = None,
) -> None:
    width, height = screen_size()
    x, y, frame_width, frame_height = window_frame()
    fps = int(os.environ.get("DEMO_WINDOW_TRANSITION_FPS", "30"))
    start_hold = float(os.environ.get("DEMO_WINDOW_TRANSITION_START_HOLD_SECONDS", "0.12"))
    fade_seconds = float(os.environ.get("DEMO_WINDOW_TRANSITION_FADE_SECONDS", "0.24"))
    fade_seconds = max(0.08, min(fade_seconds, transition_seconds / 3))
    start_hold = max(0.0, min(start_hold, max(0.0, transition_seconds - fade_seconds * 2)))
    fade_in_start = max(start_hold + fade_seconds, transition_seconds - fade_seconds)

    black_frame = STATE_DIR / "black-frame.png"
    ensure_black_frame(black_frame)
    previous_source = previous_image if previous_image and previous_image.is_file() else black_frame
    target_source = target_image if target_image and target_image.is_file() else black_frame
    filter_complex = (
        f"[0:v]scale={frame_width}:{frame_height}:force_original_aspect_ratio=decrease,"
        f"pad={frame_width}:{frame_height}:(ow-iw)/2:(oh-ih)/2:black,"
        f"format=rgba,fade=t=out:st={start_hold:.3f}:d={fade_seconds:.3f}:alpha=1,setpts=PTS-STARTPTS[previous];"
        f"[1:v]scale={frame_width}:{frame_height}:force_original_aspect_ratio=decrease,"
        f"pad={frame_width}:{frame_height}:(ow-iw)/2:(oh-ih)/2:black,"
        f"format=rgba,fade=t=in:st={fade_in_start:.3f}:d={fade_seconds:.3f}:alpha=1,setpts=PTS-STARTPTS[target];"
        f"color=c=black:s={width}x{height}:r={fps}:d={transition_seconds:.3f},format=rgba[base];"
        f"[base][previous]overlay={x}:{y}:format=auto[darkout];"
        f"[darkout][target]overlay={x}:{y}:format=auto,format=yuv420p[v]"
    )
    subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-loop",
            "1",
            "-t",
            f"{transition_seconds:.3f}",
            "-i",
            str(previous_source),
            "-loop",
            "1",
            "-t",
            f"{transition_seconds:.3f}",
            "-i",
            str(target_source),
            "-filter_complex",
            filter_complex,
            "-map",
            "[v]",
            "-r",
            str(fps),
            "-an",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-crf",
            "18",
            "-pix_fmt",
            "yuv420p",
            str(path),
        ],
        cwd=ROOT,
        env=os.environ.copy(),
        check=True,
    )


def find_ffplay_window(display: str, video_path: Path) -> str:
    fallback = ""
    deadline = time.time() + float(os.environ.get("DEMO_WINDOW_TRANSITION_START_TIMEOUT_SECONDS", "5.0"))
    while time.time() < deadline:
        for command in (
            ["xdotool", "search", "--class", "carbon-demo-transition"],
            ["xdotool", "search", "--class", "ffplay"],
            ["xdotool", "search", "--name", video_path.name],
        ):
            found = subprocess.run(
                command,
                cwd=ROOT,
                env={**os.environ, "DISPLAY": display},
                text=True,
                capture_output=True,
                check=False,
            )
            if found.returncode != 0 or not found.stdout.strip():
                continue
            ids = [line.strip() for line in found.stdout.splitlines() if line.strip().isdigit()]
            if ids:
                fallback = ids[-1]
                return fallback
        time.sleep(0.05)
    return fallback


def run_ffplay_transition(
    label: str,
    *,
    transition_seconds: float,
    target_image: Path | None = None,
    previous_image: Path | None = None,
) -> bool:
    if not shutil.which("ffplay") or not shutil.which("ffmpeg"):
        return False
    width, height = screen_size()
    display = os.environ.get("DISPLAY", os.environ.get("DEMO_DISPLAY", ":96"))
    video_path = STATE_DIR / f"transition-{int(time.time() * 1000)}.mp4"
    try:
        render_transition_video(
            video_path,
            transition_seconds=transition_seconds,
            target_image=target_image,
            previous_image=previous_image,
        )
    except Exception:
        return False
    play_env = {
        **os.environ,
        "DISPLAY": display,
        "SDL_VIDEO_X11_WMCLASS": "carbon-demo-transition",
    }
    proc = subprocess.Popen(
        [
            "ffplay",
            "-hide_banner",
            "-loglevel",
            "error",
            "-autoexit",
            "-noborder",
            "-alwaysontop",
            "-left",
            "0",
            "-top",
            "0",
            "-x",
            str(width),
            "-y",
            str(height),
            str(video_path),
        ],
        cwd=ROOT,
        env=play_env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    window_id = ""
    try:
        window_id = find_ffplay_window(display, video_path)
        time.sleep(float(os.environ.get("DEMO_WINDOW_TRANSITION_READY_DELAY_SECONDS", "0.16")))
        if window_id:
            xdo(display, "windowmove", window_id, "0", "0")
            xdo(display, "windowsize", window_id, str(width), str(height))
            xdo(display, "windowraise", window_id)
            xdo(display, "windowfocus", window_id)
        place_target_behind_blackout(display, window_id)
        if window_id:
            xdo(display, "windowraise", window_id)
            xdo(display, "windowfocus", window_id)
        mark_ready()
        try:
            proc.wait(timeout=max(2.0, transition_seconds + 3.0))
        except subprocess.TimeoutExpired:
            proc.terminate()
            proc.wait(timeout=2)
        return True
    finally:
        if proc.poll() is None:
            proc.terminate()
            try:
                proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                proc.kill()
        video_path.unlink(missing_ok=True)


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
    transition_seconds = float(os.environ.get("DEMO_WINDOW_TRANSITION_SECONDS", "1.02"))
    raw_target_image = os.environ.get("DEMO_WINDOW_TRANSITION_TARGET_IMAGE", "").strip()
    target_image = Path(raw_target_image) if raw_target_image else None
    previous_image = STATE_DIR / "previous.png"
    if run_ffplay_transition(
        label,
        transition_seconds=transition_seconds,
        target_image=target_image,
        previous_image=previous_image,
    ):
        STATE_FILE.write_text(json.dumps({"slot": slot_for_label(label), "label": label}, indent=2))
        return

    write_blackout_html(
        html_path,
        transition_seconds=transition_seconds,
        target_image=target_image,
        previous_image=previous_image,
    )

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
            "--default-background-color=000000",
            "--disable-features=CalculateNativeWinOcclusion",
            "--ozone-platform=x11",
            "--window-position=-32000,-32000",
            f"--window-size={width},{height}",
            f"--user-data-dir={profile}",
            f"--app={html_path.resolve().as_uri()}",
        ],
        cwd=ROOT,
        env={**os.environ, "DISPLAY": display},
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    try:
        window_id = find_transition_window(display, html_path)
        time.sleep(float(os.environ.get("DEMO_WINDOW_TRANSITION_READY_DELAY_SECONDS", "0.45")))
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
