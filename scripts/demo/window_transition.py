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


def transition_start_ms() -> int:
    """Delay before the transition begins, shared by the CSS timeline and the
    process-side hold computation so they cannot drift apart."""
    return int(os.environ.get("DEMO_WINDOW_TRANSITION_START_DELAY_MS", "700"))


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
    display = os.environ.get("DISPLAY", os.environ.get("DEMO_DISPLAY", ":95"))
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


def card_layout() -> list[tuple[str, str]]:
    return [
        ("Jupiter Live", "Carbon pipeline terminal"),
        ("Token Program", "Account snapshot terminal"),
        ("Grafana", "Carbon metrics dashboard"),
        ("ClickStack", "ClickHouse query telemetry"),
        ("ClickHouse Play", "Landing table SQL"),
        ("Async Inserts", "Async insert evidence"),
    ]


def slot_for_label(label: str) -> int:
    configured = os.environ.get("DEMO_TRANSITION_TARGET_SLOT")
    if configured not in {None, ""}:
        return max(0, min(len(card_layout()) - 1, int(configured)))
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


def read_state() -> dict:
    try:
        return json.loads(STATE_FILE.read_text())
    except (FileNotFoundError, json.JSONDecodeError):
        return {}


def write_state(slot: int, label: str) -> None:
    STATE_FILE.write_text(json.dumps({"slot": slot, "label": label}, indent=2))


def numeric_window_ids(raw: str) -> list[str]:
    return [line.strip() for line in raw.splitlines() if line.strip().isdigit()]


def debug(message: str) -> None:
    path = os.environ.get("DEMO_WINDOW_TRANSITION_DEBUG_LOG")
    if not path:
        return
    with Path(path).open("a") as handle:
        handle.write(f"{time.time():.3f} {message}\n")


def image_uri(path: Path) -> str:
    return path.resolve().as_uri()


def slot_image(slot: int) -> Path:
    return STATE_DIR / f"slot-{slot}.png"


def resolve_slot_image(slot: int) -> Path | None:
    """Pick the thumbnail for a non-target overview slot.

    Prefers a slot image captured during this run, then falls back to a
    pre-seeded image from ``DEMO_SLOT_SEED_DIR`` so the overview can be fully
    populated before any slot has been visited. Returns ``None`` when neither
    exists, in which case the card renders a muted placeholder.
    """
    captured = slot_image(slot)
    if captured.is_file():
        return captured
    seed_dir = os.environ.get("DEMO_SLOT_SEED_DIR")
    if seed_dir:
        seeded = Path(seed_dir) / f"slot-{slot}.png"
        if seeded.is_file():
            return seeded
    return None


def write_html(
    path: Path,
    *,
    previous: Path | None,
    target: Path,
    label: str,
    first: bool,
    previous_slot: int,
    target_slot: int,
) -> None:
    width, height = screen_size()
    margin, _, full_w, full_h = window_frame()
    overview_margin = int(os.environ.get("DEMO_OVERVIEW_MARGIN", "96"))
    gap = int(os.environ.get("DEMO_OVERVIEW_GAP", "48"))
    cols = 3
    rows = 2
    card_w = (width - overview_margin * 2 - gap * (cols - 1)) // cols
    card_h = (height - overview_margin * 2 - gap * (rows - 1)) // rows
    target_index = max(0, min(len(card_layout()) - 1, target_slot))
    previous_index = max(0, min(len(card_layout()) - 1, previous_slot))
    target_col = target_index % cols
    target_row = target_index // cols
    previous_col = previous_index % cols
    previous_row = previous_index // cols
    target_x = overview_margin + target_col * (card_w + gap)
    target_y = overview_margin + target_row * (card_h + gap)
    current_x = overview_margin + previous_col * (card_w + gap)
    current_y = overview_margin + previous_row * (card_h + gap)
    # Motion is expressed as transform: translate()+scale() on a full-size box
    # (composited, no per-frame layout/paint) instead of animating
    # left/top/width/height, which janks under software rendering. A uniform
    # scale to the card width letterboxes vertically exactly like the static
    # thumbnails, so the slot/full hand-off stays pixel-aligned.
    slot_scale = card_w / full_w
    scaled_h = full_h * slot_scale
    target_tx = target_x - margin
    target_ty = target_y + (card_h - scaled_h) / 2 - margin
    current_tx = current_x - margin
    current_ty = current_y + (card_h - scaled_h) / 2 - margin
    target_slot_transform = f"translate({target_tx:.2f}px, {target_ty:.2f}px) scale({slot_scale:.5f})"
    current_slot_transform = f"translate({current_tx:.2f}px, {current_ty:.2f}px) scale({slot_scale:.5f})"
    transition_start = transition_start_ms()
    has_current = previous is not None and not first
    previous_css = (
        f'<div class="screen current"><img src="{image_uri(previous)}" alt="previous window"></div>' if has_current else ""
    )
    target_css = f'<div class="screen target"><img src="{image_uri(target)}" alt="target window"></div>'
    cards = []
    for index, (title, subtitle) in enumerate(card_layout()):
        col = index % cols
        row = index // cols
        x = overview_margin + col * (card_w + gap)
        y = overview_margin + row * (card_h + gap)
        if index == target_index:
            cls = "card selected"
        elif has_current and index == previous_index:
            # The shrinking window lands in this slot; its thumbnail fades in
            # *after* the window has settled so there is no window->thumbnail snap.
            cls = "card fromcard"
        else:
            cls = "card"
        candidate = target if index == target_index else resolve_slot_image(index)
        image_html = (
            f'<img class="thumb" src="{image_uri(candidate)}" alt="{html.escape(title)}">'
            if candidate is not None and candidate.is_file()
            else ""
        )
        empty_cls = "" if image_html else " empty"
        cards.append(
            f'<div class="{cls}{empty_cls}" style="left:{x}px;top:{y}px;width:{card_w}px;height:{card_h}px">'
            f"{image_html}<div class=\"card-label\"><strong>{html.escape(title)}</strong><span>{html.escape(subtitle)}</span></div></div>"
        )
    # Timeline (ms after --demo-transition-start). targetIn fires at +1760ms
    # (see the .target rule). Neighbours must clear *before* the expanding
    # target grows over them, and the selected tile's label must hand off to
    # the full-window overlay without a visible step.
    fade_in = "cardsIn 420ms cubic-bezier(.2,.8,.2,1) calc(var(--demo-transition-start) + 620ms) forwards"
    neighbours_out = "cardsOut 360ms cubic-bezier(.4,0,.2,1) calc(var(--demo-transition-start) + 1480ms) forwards"
    # The current window finishes shrinking at +shrink_ms; the slot it lands in
    # only fades its thumbnail in afterwards, so the hand-off is seamless.
    shrink_ms = 850
    fromcard_in = f"cardsIn 320ms cubic-bezier(.2,.8,.2,1) calc(var(--demo-transition-start) + {shrink_ms}ms) forwards"
    previous_card_animation = f"{fromcard_in},\n    {neighbours_out}"
    # Selected label clears just as the target takes over (ends at +1760ms).
    selected_label_out = "cardsOut 180ms cubic-bezier(.4,0,.2,1) calc(var(--demo-transition-start) + 1580ms) forwards"
    # Selected tile (thumb + border) backs the slot through the hand-off, then
    # fades once the opaque target already covers it.
    selected_card_out = "cardsOut 300ms linear calc(var(--demo-transition-start) + 1840ms) forwards"
    if first:
        card_animation = neighbours_out
        card_opacity = "1"
        selected_card_animation = selected_card_out
        selected_label_animation = selected_label_out
    else:
        card_animation = f"{fade_in},\n    {neighbours_out}"
        card_opacity = "0"
        selected_card_animation = f"{fade_in},\n    {selected_card_out}"
        selected_label_animation = selected_label_out
    # Target stays hidden during the overview hold and appears at full opacity
    # over the identical selected thumb, so the hand-off has no opacity step.
    target_hold_opacity = "0"
    path.write_text(
        f"""<!doctype html>
<html style="background:#050608;color-scheme:dark;">
<head>
<meta charset="utf-8">
<meta name="color-scheme" content="dark">
<title>Carbon Window Transition</title>
<style>
html, body {{
  margin: 0;
  width: {width}px;
  height: {height}px;
  overflow: hidden;
  background: #050608;
  color: #e8edf2;
  font-family: ui-sans-serif, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}}
:root {{
  --demo-transition-start: {transition_start}ms;
}}
body {{
  background: #050608;
}}
.overview-shell {{
  position: absolute;
  inset: 0;
  background: #050608;
  opacity: {card_opacity};
  animation: {card_animation};
}}
.workspace-strip {{
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  height: 46px;
  background: #090b0f;
  border-bottom: 1px solid rgba(255,255,255,.08);
}}
.workspace-strip span {{
  display: inline-block;
  margin: 12px 0 0 {overview_margin}px;
  color: #8f9aa6;
  font-size: 14px;
  letter-spacing: .08em;
  text-transform: uppercase;
}}
.screen {{
  position: absolute;
  left: {margin}px;
  top: {margin}px;
  width: {full_w}px;
  height: {full_h}px;
  border-radius: 14px;
  overflow: hidden;
  box-shadow: 0 26px 76px rgba(0,0,0,.50), 0 0 0 1px rgba(255,255,255,.10);
  background: #0b1017;
  transform-origin: 0 0;
  will-change: transform, opacity;
}}
.screen img {{
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: #090d12;
  display: block;
}}
.current {{
  animation:
    currentOut {shrink_ms}ms cubic-bezier(.2,.72,.14,1) var(--demo-transition-start) forwards,
    cardsOut 360ms cubic-bezier(.4,0,.2,1) calc(var(--demo-transition-start) + 1480ms) forwards;
}}
.target {{
  opacity: {target_hold_opacity};
  animation:
    targetHold 1760ms linear var(--demo-transition-start) backwards,
    targetIn 1220ms cubic-bezier(.2,.72,.14,1) calc(var(--demo-transition-start) + 1760ms) forwards;
}}
.card {{
  position: absolute;
  box-sizing: border-box;
  border-radius: 14px;
  border: 1px solid rgba(255,255,255,.10);
  background: #0b1016;
  box-shadow: 0 18px 58px rgba(0,0,0,.42);
  opacity: {card_opacity};
  overflow: hidden;
  animation: {card_animation};
}}
.thumb {{
  width: 100%;
  height: 100%;
  object-fit: contain;
  background: #090d12;
  display: block;
  opacity: 1;
}}
.card-label {{
  position: absolute;
  left: 0;
  right: 0;
  top: 0;
  padding: 16px 18px 34px;
  background: linear-gradient(180deg, rgba(5,6,8,.62), rgba(5,6,8,.22) 60%, transparent);
}}
.card strong {{
  display: block;
  color: #faff69;
  font-size: 20px;
  margin-bottom: 7px;
  text-shadow: 0 1px 4px rgba(0,0,0,.85);
}}
.card span {{
  display: block;
  color: #aeb9c5;
  font-size: 14px;
  text-shadow: 0 1px 4px rgba(0,0,0,.85);
}}
.card.selected {{
  border-color: rgba(250,255,105,.55);
  box-shadow: 0 0 0 1px rgba(250,255,105,.18), 0 22px 80px rgba(0,0,0,.54);
  animation: {selected_card_animation};
}}
.card.selected .card-label {{
  animation: {selected_label_animation};
}}
.card.fromcard {{
  animation: {previous_card_animation};
}}
/* Empty slots read as dimmed, unfocused workspace windows rather than
   failed captures: faux titlebar dots, a centred muted label, no bright
   accent, and recessed so populated tiles stay dominant. */
.card.empty {{
  background: linear-gradient(165deg, #0d141d 0%, #0a0e15 100%);
  border-color: rgba(255,255,255,.06);
  box-shadow: inset 0 0 0 1px rgba(255,255,255,.02), 0 14px 44px rgba(0,0,0,.36);
  filter: brightness(.82) saturate(.9);
}}
.card.empty::before {{
  content: "";
  position: absolute;
  left: 20px;
  top: 18px;
  width: 44px;
  height: 9px;
  background:
    radial-gradient(circle 4px at 4px 4px, rgba(255,255,255,.16) 96%, transparent),
    radial-gradient(circle 4px at 22px 4px, rgba(255,255,255,.12) 96%, transparent),
    radial-gradient(circle 4px at 40px 4px, rgba(255,255,255,.09) 96%, transparent);
}}
.card.empty .card-label {{
  position: absolute;
  left: 0;
  right: 0;
  top: 50%;
  transform: translateY(-50%);
  padding: 0 26px;
  text-align: center;
  background: transparent;
}}
.card.empty strong {{
  color: #c2ccd7;
  font-weight: 600;
}}
.card.empty span {{
  color: #717d8a;
}}
@keyframes currentOut {{
  from {{ transform: none; opacity:1; }}
  to {{ transform: {current_slot_transform}; opacity:1; }}
}}
@keyframes targetIn {{
  0% {{ transform: {target_slot_transform}; opacity:1; }}
  100% {{ transform: none; opacity:1; }}
}}
@keyframes targetHold {{
  from {{ transform: {target_slot_transform}; opacity:{target_hold_opacity}; }}
  to {{ transform: {target_slot_transform}; opacity:{target_hold_opacity}; }}
}}
@keyframes cardsIn {{ to {{ opacity:1; }} }}
@keyframes cardsOut {{ to {{ opacity:0; }} }}
</style>
</head>
<body>
{previous_css}
<div class="overview-shell"><div class="workspace-strip"><span>Carbon ClickHouse tutorial</span></div></div>
{''.join(cards)}
{target_css}
</body>
</html>
"""
    )


def run_transition(label: str) -> None:
    if os.environ.get("DEMO_WINDOW_OVERVIEW", "true").lower() in {"0", "false", "no", "off"}:
        return
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    target = STATE_DIR / "target.png"
    previous = STATE_DIR / "previous.png"
    html_path = STATE_DIR / "transition.html"
    target_image = os.environ.get("DEMO_WINDOW_TRANSITION_TARGET_IMAGE")
    if target_image and Path(target_image).is_file():
        shutil.copyfile(target_image, target)
    else:
        capture_display(target)
    first = not previous.is_file()
    state = read_state()
    target_slot = slot_for_label(label)
    previous_slot = int(state.get("slot", 0))
    write_html(
        html_path,
        previous=previous if previous.is_file() else None,
        target=target,
        label=label,
        first=first,
        previous_slot=previous_slot,
        target_slot=target_slot,
    )
    width, height = screen_size()
    display = os.environ.get("DISPLAY", os.environ.get("DEMO_DISPLAY", ":95"))
    profile = STATE_DIR / f"profile-{int(time.time() * 1000)}"
    # Launch the overlay off-screen so Chromium's white window-creation paint
    # is never recorded, then move the titled transition window on-screen. This
    # is safe only because lookup below requires the stable transition title;
    # PID-based lookup is intentionally avoided on this X server.
    prepare_offscreen = os.environ.get("DEMO_TRANSITION_PREPARE_OFFSCREEN", "true").lower() not in {"0", "false", "no", "off"}
    initial_x = -32000 if prepare_offscreen else 0
    initial_y = -32000 if prepare_offscreen else 0
    transition_url = html_path.resolve().as_uri()
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
            # Keep painting even if a caller explicitly opts into off-screen
            # preparation.
            "--disable-backgrounding-occluded-windows",
            "--disable-renderer-backgrounding",
            "--disable-features=CalculateNativeWinOcclusion",
            "--ozone-platform=x11",
            f"--window-position={initial_x},{initial_y}",
            f"--window-size={width},{height}",
            f"--user-data-dir={profile}",
            f"--app={transition_url}",
        ],
        cwd=ROOT,
        env={**os.environ, "DISPLAY": display},
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    min_transition_seconds = (transition_start_ms() + 1760 + 1220) / 1000 + 0.35
    requested_transition_seconds = float(os.environ.get("DEMO_WINDOW_TRANSITION_SECONDS", "3.4"))
    transition_seconds = max(requested_transition_seconds, min_transition_seconds)
    debug(f"launched pid={proc.pid} prepare_offscreen={prepare_offscreen} transition_seconds={transition_seconds:.3f}")
    try:
        deadline = time.time() + float(os.environ.get("DEMO_WINDOW_TRANSITION_START_TIMEOUT_SECONDS", "5.0"))
        window_id = ""
        while time.time() < deadline:
            by_name = subprocess.run(
                ["xdotool", "search", "--name", "Carbon Window Transition"],
                cwd=ROOT,
                env={**os.environ, "DISPLAY": display},
                text=True,
                capture_output=True,
            )
            if by_name.returncode == 0 and by_name.stdout.strip():
                ids = numeric_window_ids(by_name.stdout)
                if ids:
                    window_id = ids[-1]
                    debug(f"selected by_name window_id={window_id} ids={ids}")
                    break
            found = subprocess.run(
                ["xdotool", "search", "--class", "chrome"],
                cwd=ROOT,
                env={**os.environ, "DISPLAY": display},
                text=True,
                capture_output=True,
            )
            if found.returncode == 0 and found.stdout.strip():
                ids = numeric_window_ids(found.stdout)
                for candidate in reversed(ids):
                    title = subprocess.run(
                        ["xdotool", "getwindowname", candidate],
                        cwd=ROOT,
                        env={**os.environ, "DISPLAY": display},
                        text=True,
                        capture_output=True,
                    )
                    if (
                        "Carbon Window Transition" in title.stdout
                        or "transition.html" in title.stdout
                        or str(html_path.name) in title.stdout
                    ):
                        window_id = candidate
                        debug(f"selected by_class window_id={window_id} ids={ids} title={title.stdout.strip()!r}")
                        break
                if window_id:
                    break
            time.sleep(0.05)
        time.sleep(float(os.environ.get("DEMO_WINDOW_TRANSITION_READY_DELAY_SECONDS", "0.6")))
        if not window_id:
            raise RuntimeError("transition overlay window was not found")
        if window_id:
            debug(f"raising window_id={window_id}")
            subprocess.run(
                ["xdotool", "windowmove", window_id, "0", "0"],
                cwd=ROOT,
                env={**os.environ, "DISPLAY": display},
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
            subprocess.run(
                ["xdotool", "windowsize", window_id, str(width), str(height)],
                cwd=ROOT,
                env={**os.environ, "DISPLAY": display},
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
            subprocess.run(
                ["xdotool", "windowfocus", window_id],
                cwd=ROOT,
                env={**os.environ, "DISPLAY": display},
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
            subprocess.run(
                ["xdotool", "windowraise", window_id],
                cwd=ROOT,
                env={**os.environ, "DISPLAY": display},
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
            # Place the real target window at the single-window frame *behind*
            # the overlay (which now fully covers the screen). The window keeps
            # its natural stacking -- above the terminal, below the overlay --
            # so it stays hidden until the overlay terminates and is then
            # revealed with no gap. Callers therefore no longer need to move the
            # target on-screen before the transition, which is what produced the
            # premature target/terminal flashes.
            target_window_id = os.environ.get("DEMO_WINDOW_TRANSITION_TARGET_WINDOW_ID", "").strip()
            if target_window_id:
                tx, ty, tw, th = window_frame()
                for cmd in (
                    ["windowmove", target_window_id, str(tx), str(ty)],
                    ["windowsize", target_window_id, str(tw), str(th)],
                    ["windowraise", window_id],
                ):
                    subprocess.run(
                        ["xdotool", *cmd],
                        cwd=ROOT,
                        env={**os.environ, "DISPLAY": display},
                        stdout=subprocess.DEVNULL,
                        stderr=subprocess.DEVNULL,
                        check=False,
                    )
        ready_file = os.environ.get("DEMO_WINDOW_TRANSITION_READY_FILE")
        if ready_file:
            Path(ready_file).write_text(f"{time.time()}\n")
        debug("transition ready; sleeping")
        time.sleep(transition_seconds)
        if window_id:
            exists = subprocess.run(
                ["xdotool", "getwindowname", window_id],
                cwd=ROOT,
                env={**os.environ, "DISPLAY": display},
                text=True,
                capture_output=True,
            )
            debug(f"post-sleep window returncode={exists.returncode} title={exists.stdout.strip()!r}")
    finally:
        debug("terminating chrome")
        proc.terminate()
        try:
            proc.wait(timeout=4)
        except subprocess.TimeoutExpired:
            proc.kill()
        shutil.rmtree(profile, ignore_errors=True)
    shutil.copyfile(target, previous)
    shutil.copyfile(target, slot_image(target_slot))
    write_state(target_slot, label)


def capture_current_state(label: str) -> None:
    STATE_DIR.mkdir(parents=True, exist_ok=True)
    slot = slot_for_label(label)
    capture_display(slot_image(slot))
    shutil.copyfile(slot_image(slot), STATE_DIR / "previous.png")
    write_state(slot, label)


def reset() -> None:
    shutil.rmtree(STATE_DIR, ignore_errors=True)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("label", nargs="?")
    parser.add_argument("--reset", action="store_true")
    parser.add_argument(
        "--capture-slot",
        type=int,
        metavar="N",
        help="capture the current display into overview slot N and exit (pre-seed a tile)",
    )
    parser.add_argument(
        "--seed-slot",
        nargs=2,
        metavar=("N", "IMAGE"),
        help="copy IMAGE into overview slot N and exit (pre-seed a tile)",
    )
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
    if args.capture_slot is not None:
        STATE_DIR.mkdir(parents=True, exist_ok=True)
        capture_display(slot_image(args.capture_slot))
        return 0
    if args.seed_slot is not None:
        slot_str, image = args.seed_slot
        STATE_DIR.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(image, slot_image(int(slot_str)))
        return 0
    if not args.label:
        parser.error("label is required unless --reset/--capture-slot/--seed-slot/--capture-current is used")
    run_transition(args.label)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
