#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parents[2]
SCENES = [
    "scene-01-intro",
    "scene-02-local-setup",
    "scene-03-jupiter-ingestion",
    "scene-04-clickhouse-validation",
    "scene-05-token-program",
    "scene-06-async-inserts",
    "scene-07-observability",
    "scene-08-production-boundaries",
]

LAST_VIEW_KEY: str | None = None

TRANSITION_SEED_SOURCES = {
    2: (
        "scene-05-token-program-grafana-live-examples.png",
        "scene-05-token-program-grafana-live-examples-raw.png",
    ),
    3: (
        "scene-05-token-program-clickstack-query-log-live-examples.png",
        "scene-07-observability-clickstack-query-log-after-async.png",
    ),
    4: (
        "scene-06-async-inserts-clickhouse-play-jupiter-sample.png",
        "scene-06-async-inserts-clickhouse-play-token-sample.png",
    ),
    5: (
        "scene-07-observability-clickhouse-play-async-log.png",
    ),
}


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
    data.setdefault("RECORDER_BACKEND", "ffmpeg_x11")
    data.setdefault("DEMO_FPS", "60")
    data.setdefault("DEMO_DISPLAY", ":95")
    data.setdefault("DISPLAY", data["DEMO_DISPLAY"])
    data.setdefault("DEMO_SCREEN_SIZE", "1920x1080")
    data.setdefault("DEMO_VNC_PORT", "5903")
    data.setdefault("DEMO_NOVNC_PORT", "6083")
    data.setdefault("DEMO_SKIP_NOVNC", "true")
    return data


def preserve_transition_seed_sources() -> tuple[Path, list[int]]:
    """Preserve useful browser thumbnails before review cleanup removes screenshots.

    The transition renderer can show graceful placeholders, but prior good
    browser captures make the overview read as a real workspace immediately.
    This is opportunistic: missing seeds are fine and fall back to placeholders.
    """
    seed_dir = ROOT / "demo-artifacts/window-transition-seeds"
    seed_dir.mkdir(parents=True, exist_ok=True)
    for stale in seed_dir.glob("slot-*.png"):
        stale.unlink()

    roots = [
        ROOT / "demo-artifacts/screenshots",
        ROOT / "demo-artifacts/review-human/screenshots",
    ]
    seeded: list[int] = []
    for slot, names in TRANSITION_SEED_SOURCES.items():
        for root in roots:
            for name in names:
                candidate = root / name
                if candidate.is_file():
                    shutil.copy2(candidate, seed_dir / f"slot-{slot}.png")
                    seeded.append(slot)
                    break
            if slot in seeded:
                break
    return seed_dir, sorted(set(seeded))


def run(cmd: list[str], *, check: bool = True, log: Path | None = None, extra_env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    merged = env()
    if extra_env:
        merged.update(extra_env)
    if log:
        log.parent.mkdir(parents=True, exist_ok=True)
        with log.open("a") as handle:
            handle.write(f"$ {' '.join(cmd)}\n")
            proc = subprocess.run(cmd, cwd=ROOT, env=merged, text=True, stdout=handle, stderr=subprocess.STDOUT)
    else:
        proc = subprocess.run(cmd, cwd=ROOT, env=merged, text=True, capture_output=True)
    if check and proc.returncode != 0:
        raise subprocess.CalledProcessError(proc.returncode, cmd)
    return proc


def runbook() -> dict[str, Any]:
    return yaml.safe_load((ROOT / "scripts/demo/runbook.yaml").read_text())


def scene_map() -> dict[str, dict[str, Any]]:
    return {scene["id"]: scene for scene in runbook()["scenes"]}


def prepare_review_dir(review_dir: Path) -> str:
    timestamp = time.strftime("%Y%m%d-%H%M%S")
    if review_dir.exists():
        for name in ("videos", "screenshots", "logs", "archive"):
            path = review_dir / name
            if path.exists():
                shutil.rmtree(path)
    screenshot_source = ROOT / "demo-artifacts/screenshots"
    if screenshot_source.exists() and any(screenshot_source.glob("*.png")):
        for path in screenshot_source.glob("*.png"):
            path.unlink()
    for subdir in ("videos", "screenshots", "logs"):
        (review_dir / subdir).mkdir(parents=True, exist_ok=True)
    (review_dir / "latest-review-timestamp.txt").write_text(timestamp + "\n")
    return timestamp


def write_scene_scripts(review_dir: Path) -> None:
    with (review_dir / "scene-scripts.md").open("w") as out:
        out.write("# ClickHouse Sink Tutorial Scene Scripts\n")
        for scene in SCENES:
            path = ROOT / "docs/tutorial-video" / f"{scene}.md"
            out.write(f"\n## {scene}\n\n")
            out.write(path.read_text())
            out.write("\n\n---\n")


def write_manifest_start(review_dir: Path, timestamp: str) -> None:
    status = run(["git", "status", "--short"]).stdout
    head = run(["git", "rev-parse", "--short", "HEAD"]).stdout.strip()
    aligned = run(["git", "rev-list", "--left-right", "--count", "HEAD...origin/clickhouse-upstream-v1"]).stdout.strip()
    (review_dir / "manifest.md").write_text(
        "\n".join(
            [
                "# Carbon ClickHouse Human-Style Visual Rehearsal",
                "",
                f"- Review timestamp: {timestamp}",
                f"- HEAD: {head}",
                f"- Alignment with origin/clickhouse-upstream-v1: {aligned}",
                "- Recording backend: ffmpeg_x11",
                "- Voice generation: skipped",
                "- Audio assembly: skipped",
                "- Grafana: skipped unless local credentials are explicitly verified",
                "",
                "## Git Status",
                "",
                "```text",
                status.rstrip(),
                "```",
                "",
            ]
        )
    )


def append_manifest(review_dir: Path, text: str) -> None:
    with (review_dir / "manifest.md").open("a") as out:
        out.write(text.rstrip() + "\n\n")


def display_reachable(display_id: str) -> bool:
    return subprocess.run(
        ["xdpyinfo", "-display", display_id],
        cwd=ROOT,
        env=os.environ.copy(),
        text=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    ).returncode == 0


def start_display(log_dir: Path) -> bool:
    display_id = env().get("DEMO_DISPLAY", ":95")
    if os.environ.get("DEMO_REUSE_EXTERNAL_DISPLAY", "true").lower() not in {"0", "false", "no", "off"} and display_reachable(display_id):
        display_env = f"DEMO_DISPLAY={display_id}\nDISPLAY={display_id}\nDEMO_SCREEN_SIZE={env().get('DEMO_SCREEN_SIZE', '1920x1080')}\n"
        append_manifest(
            ROOT / "demo-artifacts/review-human",
            "## Display\n\n```env\n" + display_env.strip() + "\n```\n\n- Existing display reused; display cleanup skipped.",
        )
        return False
    run(["scripts/demo/start-display.sh"], log=log_dir / "start-display.log")
    display_env = (ROOT / "demo-artifacts/display.env").read_text()
    append_manifest(ROOT / "demo-artifacts/review-human", "## Display\n\n```env\n" + display_env.strip() + "\n```")
    return True


def close_human_terminal(log_dir: Path) -> None:
    run([sys.executable, "scripts/demo/human_scene_driver.py", "--close-terminal"], check=False, log=log_dir / "close-human-terminal.log")
    run([sys.executable, "scripts/demo/window_transition.py", "--reset"], check=False, log=log_dir / "window-transition-reset.log")


def maybe_reset(log_dir: Path) -> bool:
    run(["scripts/demo/detect-running-demo-processes.sh"], check=False, log=log_dir / "process-detection-before-reset.log")
    if os.environ.get("DEMO_ALLOW_CLICKHOUSE_RESET") != "true":
        append_manifest(ROOT / "demo-artifacts/review-human", "## Reset\n\n- ClickHouse tutorial reset performed: no (`DEMO_ALLOW_CLICKHOUSE_RESET` was not true).")
        return False
    run(["scripts/demo/reset-clickhouse.sh"], log=log_dir / "clickhouse-reset.log", extra_env={"DEMO_ALLOW_CLICKHOUSE_RESET": "true"})
    # Prometheus TSDB is not wiped by the ClickHouse reset; flush prior-run
    # series so Grafana shows only this run's metrics.
    run(["scripts/demo/reset-prometheus.sh"], check=False, log=log_dir / "prometheus-reset.log", extra_env={"DEMO_ALLOW_CLICKHOUSE_RESET": "true"})
    append_manifest(ROOT / "demo-artifacts/review-human", "## Reset\n\n- ClickHouse tutorial reset performed: yes\n- ClickHouse system logs flushed: yes\n- Prometheus series flushed: yes\n- Reset logs: `demo-artifacts/review-human/logs/clickhouse-reset.log`, `demo-artifacts/review-human/logs/prometheus-reset.log`")
    return True


def start_record(scene: str) -> None:
    recordings = ROOT / "demo-artifacts/recordings"
    for suffix in (".mp4", ".mkv", ".recording.json", ".ffmpeg.log"):
        path = recordings / f"{scene}{suffix}"
        if path.exists():
            path.unlink()
    run([sys.executable, "scripts/demo/record.py", "start", "--scene", scene])


def stop_record(scene: str) -> None:
    run([sys.executable, "scripts/demo/record.py", "stop", "--scene", scene], check=False)


def capture_transition_current(label: str, log: Path) -> None:
    run([sys.executable, "scripts/demo/window_transition.py", "--capture-current", label], check=False, log=log)


def run_transition_before_scene(scene: str, label: str, log: Path) -> None:
    ready_file = ROOT / "demo-artifacts/pids" / f"{scene}-transition-ready"
    ready_file.parent.mkdir(parents=True, exist_ok=True)
    if ready_file.exists():
        ready_file.unlink()
    merged = env()
    merged["DEMO_WINDOW_TRANSITION_READY_FILE"] = str(ready_file)
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("a") as handle:
        handle.write(f"$ {sys.executable} scripts/demo/window_transition.py {label}\n")
        proc = subprocess.Popen(
            [sys.executable, "scripts/demo/window_transition.py", label],
            cwd=ROOT,
            env=merged,
            text=True,
            stdout=handle,
            stderr=subprocess.STDOUT,
        )
        deadline = time.time() + 8
        while time.time() < deadline and proc.poll() is None:
            if ready_file.exists():
                break
            time.sleep(0.05)
        if not ready_file.exists():
            proc.terminate()
            try:
                proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                proc.kill()
            raise RuntimeError(f"transition did not become ready for {scene}")
        start_record(scene)
        return_code = proc.wait()
        if return_code != 0:
            raise subprocess.CalledProcessError(return_code, [sys.executable, "scripts/demo/window_transition.py", label])


def run_slide_with_delayed_record(scene: str, slide: str, seconds: float, log: Path) -> None:
    # Prepare the slide off-screen and only start recording once it is painted
    # and on-screen, so the leftover terminal and the page's first paint are
    # never captured before the slide appears.
    ready_file = ROOT / "demo-artifacts/pids" / f"{scene}-slide-ready"
    ready_file.parent.mkdir(parents=True, exist_ok=True)
    if ready_file.exists():
        ready_file.unlink()
    merged = env()
    merged["DEMO_SLIDE_READY_FILE"] = str(ready_file)
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("a") as handle:
        handle.write(f"$ {sys.executable} scripts/demo/browser_scene_driver.py slide {scene} {slide}\n")
        proc = subprocess.Popen(
            [sys.executable, "scripts/demo/browser_scene_driver.py", "slide", scene, slide, "--seconds", str(seconds)],
            cwd=ROOT,
            env=merged,
            text=True,
            stdout=handle,
            stderr=subprocess.STDOUT,
        )
        deadline = time.time() + 15
        while time.time() < deadline and proc.poll() is None:
            if ready_file.exists():
                break
            time.sleep(0.1)
        # Start recording once the slide signalled readiness (or as a fallback
        # if it exited/timed out, so the scene is never silently dropped).
        start_record(scene)
        return_code = proc.wait()
        if return_code != 0:
            raise subprocess.CalledProcessError(return_code, [sys.executable, "scripts/demo/browser_scene_driver.py", "slide", scene, slide])


def run_browser_workflow_with_delayed_record(scene: str, workflow: str, log: Path) -> None:
    ready_file = ROOT / "demo-artifacts/pids" / f"{scene}-browser-transition-ready"
    ready_file.parent.mkdir(parents=True, exist_ok=True)
    if ready_file.exists():
        ready_file.unlink()
    merged = env()
    merged["DEMO_WINDOW_TRANSITION_READY_FILE"] = str(ready_file)
    log.parent.mkdir(parents=True, exist_ok=True)
    with log.open("a") as handle:
        handle.write(f"$ node scripts/demo/browser_workflow.js {scene} {workflow}\n")
        proc = subprocess.Popen(
            ["node", "scripts/demo/browser_workflow.js", scene, workflow],
            cwd=ROOT,
            env=merged,
            text=True,
            stdout=handle,
            stderr=subprocess.STDOUT,
        )
        deadline = time.time() + 20
        while time.time() < deadline and proc.poll() is None:
            if ready_file.exists():
                break
            time.sleep(0.1)
        if not ready_file.exists():
            proc.terminate()
            try:
                proc.wait(timeout=4)
            except subprocess.TimeoutExpired:
                proc.kill()
            raise RuntimeError(f"browser transition did not become ready for {scene}")
        start_record(scene)
        return_code = proc.wait()
        if return_code != 0:
            raise subprocess.CalledProcessError(return_code, ["node", "scripts/demo/browser_workflow.js", scene, workflow])


def run_playwright(spec: str, extra_env: dict[str, str], log: Path) -> None:
    command = [
        "npm",
        "--prefix",
        str(ROOT / "scripts/demo/playwright"),
        "exec",
        "playwright",
        "test",
        Path(spec).name,
        "--headed",
        "--config",
        "playwright.config.ts",
    ]
    run(command, log=log, extra_env=extra_env)


def run_scene(scene: dict[str, Any], review_dir: Path) -> dict[str, Any]:
    global LAST_VIEW_KEY
    scene_id = scene["id"]
    human = scene.get("human") or {}
    kind = human.get("type")
    log_dir = review_dir / "logs"
    started = time.time()
    report = {"scene": scene_id, "kind": kind, "status": "ok", "started_at": started}
    view_key = f"{kind}:{scene_id}"
    if kind == "terminal":
        view_key = f"terminal:{human.get('terminal_name') or scene_id}"
    elif kind == "mixed_async":
        view_key = f"mixed:{human.get('terminal_name') or scene_id}:{human.get('workflow', 'async-log')}"
    elif kind == "browser":
        view_key = f"browser:{human.get('workflow') or scene_id}"
    try:
        if kind in {"terminal", "mixed_async"}:
            run(
                [sys.executable, "scripts/demo/human_scene_driver.py", "--prepare-terminal", scene_id],
                log=log_dir / f"{scene_id}-prepare.log",
                extra_env={
                    "DEMO_REUSE_TERMINAL": "true" if kind == "terminal" else "false",
                    "DEMO_WINDOW_OVERVIEW": "false",
                },
            )
        if not (kind in {"terminal", "mixed_async", "slide"} or (kind == "browser" and human.get("workflow"))):
            start_record(scene_id)
        try:
            if kind == "terminal":
                transition_label = str(human.get("terminal_title") or scene.get("title") or scene_id)
                same_terminal = LAST_VIEW_KEY == view_key
                if same_terminal:
                    start_record(scene_id)
                else:
                    run_transition_before_scene(
                        scene_id,
                        transition_label,
                        log_dir / f"{scene_id}-transition.log",
                    )
                run(
                    [sys.executable, "scripts/demo/human_scene_driver.py", scene_id],
                    log=log_dir / f"{scene_id}.log",
                    extra_env={
                        "DEMO_REUSE_TERMINAL": "true",
                        "DEMO_WINDOW_OVERVIEW": "false",
                        "DEMO_SUPPRESS_DRIVER_TRANSITION": "true",
                    },
                )
                capture_transition_current(transition_label, log_dir / f"{scene_id}-transition-state.log")
            elif kind == "slide":
                cleanup_after = human.get("cleanup_live_terminals_after_seconds")
                if cleanup_after is not None:
                    subprocess.Popen(
                        [
                            "bash",
                            "-lc",
                            f"sleep {float(cleanup_after)}; {sys.executable} scripts/demo/human_scene_driver.py --close-terminal",
                        ],
                        cwd=ROOT,
                        env=env(),
                        stdout=(log_dir / f"{scene_id}-background-cleanup.log").open("a"),
                        stderr=subprocess.STDOUT,
                    )
                run_slide_with_delayed_record(
                    scene_id,
                    str(human["slide"]),
                    float(human.get("duration", 10)),
                    log_dir / f"{scene_id}.log",
                )
            elif kind == "browser":
                workflow = human.get("workflow")
                if workflow:
                    run_browser_workflow_with_delayed_record(scene_id, str(workflow), log_dir / f"{scene_id}.log")
                else:
                    run(["scripts/demo/validate-observability-data.sh", str(log_dir)], log=log_dir / "observability-data.log")
                    query = (log_dir / "prometheus-query.txt").read_text().strip()
                    run(
                        [
                            sys.executable,
                            "scripts/demo/browser_scene_driver.py",
                            "prometheus",
                            scene_id,
                            query,
                            "--seconds",
                            str(int(human.get("duration", human.get("min_duration", 12)))),
                        ],
                        log=log_dir / f"{scene_id}.log",
                    )
            elif kind == "mixed_async":
                transition_label = str(human.get("terminal_title") or scene.get("title") or scene_id)
                run_transition_before_scene(
                    scene_id,
                    transition_label,
                    log_dir / f"{scene_id}-transition.log",
                )
                run(
                    [sys.executable, "scripts/demo/human_scene_driver.py", scene_id],
                    log=log_dir / f"{scene_id}-terminal.log",
                    extra_env={
                        "DEMO_REUSE_TERMINAL": "false",
                        "DEMO_WINDOW_OVERVIEW": "false",
                        "DEMO_SUPPRESS_DRIVER_TRANSITION": "true",
                    },
                )
                capture_transition_current(transition_label, log_dir / f"{scene_id}-transition-state.log")
                run(
                    ["node", "scripts/demo/browser_workflow.js", scene_id, str(human.get("workflow", "async-log"))],
                    log=log_dir / f"{scene_id}-browser.log",
                )
            else:
                raise RuntimeError(f"unsupported human scene type: {kind}")
        finally:
            stop_record(scene_id)
    except Exception as exc:
        report["status"] = "failed"
        report["error"] = str(exc)
    if report["status"] == "ok":
        LAST_VIEW_KEY = view_key
    report["finished_at"] = time.time()
    (review_dir / "logs" / f"{scene_id}.report.json").write_text(json.dumps(report, indent=2))
    return report


def normalize_video(scene: str, review_dir: Path) -> None:
    source = ROOT / "demo-artifacts/recordings" / f"{scene}.mp4"
    if not source.is_file():
        source = ROOT / "demo-artifacts/recordings" / f"{scene}.mkv"
    if not source.is_file():
        raise FileNotFoundError(source)
    target = review_dir / "videos" / f"{scene}.mp4"
    tail_trim_seconds = {
        "scene-05-token-program": 0.8,
        "scene-06-async-inserts": 0.8,
        "scene-07-observability": 2.0,
    }.get(scene, 0.0)
    needs_filter = scene in {"scene-01-intro", "scene-02-local-setup", "scene-07-observability", "scene-08-production-boundaries"} or tail_trim_seconds > 0
    if needs_filter:
        duration = float(
            subprocess.check_output(
                [
                    "ffprobe",
                    "-v",
                    "error",
                    "-show_entries",
                    "format=duration",
                    "-of",
                    "default=nw=1:nk=1",
                    str(source),
                ],
                cwd=ROOT,
                text=True,
            ).strip()
        )
        filters: list[str] = []
        output_duration = max(0.1, duration - tail_trim_seconds)
        if tail_trim_seconds > 0:
            filters.append(f"trim=end={output_duration:.3f}")
            filters.append("setpts=PTS-STARTPTS")
        if scene in {"scene-01-intro", "scene-07-observability"}:
            filters.append(f"fade=t=out:st={max(0.0, output_duration - 0.8):.3f}:d=0.8")
        if scene in {"scene-02-local-setup", "scene-08-production-boundaries"}:
            filters.append("fade=t=in:st=0:d=0.35")
        vf = ",".join(filters)
        subprocess.run(
            [
                "ffmpeg",
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-i",
                str(source),
                "-map",
                "0:v:0",
                "-an",
                "-vf",
                vf,
                "-c:v",
                "libx264",
                "-preset",
                "veryfast",
                "-pix_fmt",
                "yuv420p",
                str(target),
            ],
            cwd=ROOT,
            check=True,
        )
        return
    command = ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-i", str(source), "-map", "0:v:0", "-an", "-c:v", "copy", str(target)]
    proc = subprocess.run(command, cwd=ROOT)
    if proc.returncode != 0:
        subprocess.run(
            [
                "ffmpeg",
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-i",
                str(source),
                "-map",
                "0:v:0",
                "-an",
                "-c:v",
                "libx264",
                "-preset",
                "veryfast",
                "-pix_fmt",
                "yuv420p",
                str(target),
            ],
            cwd=ROOT,
            check=True,
        )


def concat_final(review_dir: Path) -> None:
    concat = review_dir / "videos/concat.txt"
    concat.write_text("".join(f"file '{scene}.mp4'\n" for scene in SCENES))
    output = review_dir / "videos/clickhouse-sink-tutorial-human-no-audio.mp4"
    proc = subprocess.run(["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "concat", "-safe", "0", "-i", str(concat.name), "-c", "copy", output.name], cwd=review_dir / "videos")
    if proc.returncode != 0:
        subprocess.run(
            ["ffmpeg", "-hide_banner", "-loglevel", "error", "-y", "-f", "concat", "-safe", "0", "-i", str(concat.name), "-c:v", "libx264", "-preset", "veryfast", "-pix_fmt", "yuv420p", "-an", output.name],
            cwd=review_dir / "videos",
            check=True,
        )


def copy_screenshots(review_dir: Path) -> None:
    source = ROOT / "demo-artifacts/screenshots"
    target = review_dir / "screenshots"
    target.mkdir(parents=True, exist_ok=True)
    if source.is_dir():
        for path in source.glob("*.png"):
            shutil.copy2(path, target / path.name)


def post_run_summary(review_dir: Path) -> None:
    log = review_dir / "logs/post-run-clickhouse-summary.log"
    with log.open("w") as handle:
        for label, command in [
            ("ClickHouse health", ["scripts/demo/query-clickhouse.sh", "health"]),
            ("Jupiter", ["scripts/demo/query-clickhouse.sh", "jupiter"]),
            ("Token Program", ["scripts/demo/query-clickhouse.sh", "token"]),
            ("Async log", ["scripts/demo/query-clickhouse.sh", "async-log"]),
        ]:
            handle.write(f"## {label}\n")
            proc = subprocess.run(command, cwd=ROOT, env=env(), text=True, stdout=handle, stderr=subprocess.STDOUT)
            handle.write(f"\n(exit {proc.returncode})\n\n")
    append_manifest(review_dir, "## Post-Run Data Summary\n\n```text\n" + log.read_text() + "```")


def generate_subtitles(review_dir: Path) -> None:
    run([sys.executable, "scripts/demo/subtitles.py"], log=review_dir / "logs/subtitles.log")
    append_manifest(
        review_dir,
        "\n".join(
            [
                "## Subtitles",
                "",
                "- SRT: `demo-artifacts/review-human/subtitles/clickhouse-sink-tutorial-human-no-audio.srt`",
                "- WebVTT: `demo-artifacts/review-human/subtitles/clickhouse-sink-tutorial-human-no-audio.vtt`",
                "- Burned-in review video: `demo-artifacts/review-human/videos/clickhouse-sink-tutorial-human-no-audio-subtitled.mp4`",
            ]
        ),
    )


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser()
    parser.add_argument("--no-voice", action="store_true", help="Accepted for parity; this command never calls ElevenLabs.")
    parser.add_argument("--skip-reset", action="store_true")
    args = parser.parse_args()

    seed_dir, seeded_slots = preserve_transition_seed_sources()
    os.environ.update(env())
    os.environ.setdefault("DEMO_SLOT_SEED_DIR", str(seed_dir))
    review_dir = ROOT / "demo-artifacts/review-human"
    timestamp = prepare_review_dir(review_dir)
    write_scene_scripts(review_dir)
    shutil.copy2(ROOT / "scripts/demo/runbook.yaml", review_dir / "runbook.yaml")
    write_manifest_start(review_dir, timestamp)
    if seeded_slots:
        append_manifest(
            review_dir,
            "## Window Transition Seeds\n\n"
            + f"- Seed directory: `{seed_dir.relative_to(ROOT)}`\n"
            + "- Seeded slots: "
            + ", ".join(str(slot) for slot in seeded_slots),
        )
    else:
        append_manifest(
            review_dir,
            "## Window Transition Seeds\n\n"
            + f"- Seed directory: `{seed_dir.relative_to(ROOT)}`\n"
            + "- Seeded slots: none; transition overview will use built-in placeholders until each slot is visited.",
        )

    reports: list[dict[str, Any]] = []
    scenes = scene_map()
    run(["scripts/demo/preflight.sh", "--no-elevenlabs", "--no-obs"], log=review_dir / "logs/preflight.log")
    display_started = False
    try:
        display_started = start_display(review_dir / "logs")
        close_human_terminal(review_dir / "logs")
        if not args.skip_reset:
            maybe_reset(review_dir / "logs")
        completed_scenes: list[str] = []
        for scene_id in SCENES:
            report = run_scene(scenes[scene_id], review_dir)
            reports.append(report)
            if report["status"] != "ok":
                break
            completed_scenes.append(scene_id)
        failed = [item for item in reports if item["status"] != "ok"]
        if failed:
            for scene_id in completed_scenes:
                try:
                    normalize_video(scene_id, review_dir)
                except Exception as exc:
                    append_manifest(review_dir, f"## Partial Video Export Warning\n\n- `{scene_id}` could not be normalized after failure: {exc}")
            append_manifest(
                review_dir,
                "## Scene Results\n\n" + "\n".join(f"- {item['scene']}: {item['status']} ({item.get('kind')})" for item in reports),
            )
            return 1
        for scene_id in completed_scenes:
            normalize_video(scene_id, review_dir)
        concat_final(review_dir)
        copy_screenshots(review_dir)
        post_run_summary(review_dir)
        validation = run([sys.executable, "scripts/demo/validate-video.py"], log=review_dir / "logs/video-validation.log")
        generate_subtitles(review_dir)
        run(["scripts/demo/export-review-bundle.sh", "--no-archive", str(review_dir)])
        append_manifest(review_dir, "## Review Archive\n\n- Archive creation: skipped for the current review workflow.")
    finally:
        close_human_terminal(review_dir / "logs")
        if display_started:
            run(["scripts/demo/stop-display.sh"], check=False, log=review_dir / "logs/stop-display.log")
        else:
            append_manifest(review_dir, "## Display Cleanup\n\n- Existing display was reused and left running.")

    append_manifest(
        review_dir,
        "## Scene Results\n\n" + "\n".join(f"- {item['scene']}: {item['status']} ({item.get('kind')})" for item in reports),
    )
    failed = [item for item in reports if item["status"] != "ok"]
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
