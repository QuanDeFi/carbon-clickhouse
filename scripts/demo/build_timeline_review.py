#!/usr/bin/env python3
from __future__ import annotations

import html
import json
import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml


ROOT = Path(__file__).resolve().parents[2]
REVIEW_DIR = ROOT / "demo-artifacts/review-human"
VIDEO_DIR = REVIEW_DIR / "videos"
SUBTITLE_DIR = REVIEW_DIR / "subtitles"
LOG_DIR = REVIEW_DIR / "logs"
ASSET_DIR = REVIEW_DIR / "timeline-assets"
AUDIO_ANALYSIS_DIR = REVIEW_DIR / "audio-analysis"
VOICEOVER_REPORT = REVIEW_DIR / "audio/voiceover-placement-report.json"
OUTPUT = REVIEW_DIR / "subtitle-alignment-timeline.html"
VIDEO = VIDEO_DIR / "clickhouse-sink-tutorial-human-no-audio-subtitled.mp4"
RAW_VIDEO = VIDEO_DIR / "clickhouse-sink-tutorial-human-no-audio.mp4"
VTT = SUBTITLE_DIR / "clickhouse-sink-tutorial-human-no-audio.vtt"
SRT = SUBTITLE_DIR / "clickhouse-sink-tutorial-human-no-audio.srt"
SUBTITLE_REPORT = SUBTITLE_DIR / "subtitle-report.json"
TIMING_VALIDATION = LOG_DIR / "timing-validation.json"
RUNBOOK = ROOT / "scripts/demo/runbook.yaml"


@dataclass(frozen=True)
class Cue:
    index: int
    start: float
    end: float
    text: str
    scene: str


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def load_runbook() -> dict[str, Any]:
    return yaml.safe_load(RUNBOOK.read_text())


def parse_srt_time(value: str) -> float:
    hours, minutes, rest = value.split(":")
    seconds, millis = rest.split(",")
    return int(hours) * 3600 + int(minutes) * 60 + int(seconds) + int(millis) / 1000


def fmt_time(seconds: float) -> str:
    seconds = max(0.0, seconds)
    minutes, sec = divmod(seconds, 60)
    return f"{int(minutes):02d}:{sec:05.2f}"


def pct(value: float, total: float) -> str:
    if total <= 0:
        return "0%"
    return f"{max(0.0, min(100.0, value / total * 100.0)):.4f}%"


def parse_srt(path: Path, scenes: list[dict[str, Any]]) -> list[Cue]:
    cues: list[Cue] = []
    for block in path.read_text().strip().split("\n\n"):
        lines = [line.rstrip() for line in block.splitlines() if line.strip()]
        if len(lines) < 3 or "-->" not in lines[1]:
            continue
        index = int(lines[0])
        raw_start, raw_end = [part.strip() for part in lines[1].split("-->", 1)]
        start = parse_srt_time(raw_start)
        end = parse_srt_time(raw_end)
        text = " ".join(line.strip() for line in lines[2:])
        scene = scene_for_time(scenes, start)["scene"]
        cues.append(Cue(index=index, start=start, end=end, text=text, scene=scene))
    return cues


def scene_for_time(scenes: list[dict[str, Any]], seconds: float) -> dict[str, Any]:
    for scene in scenes:
        start = float(scene["start_seconds"])
        end = start + float(scene["duration_seconds"])
        if start <= seconds < end:
            return scene
    return scenes[-1]


def rel(path: Path) -> str:
    return html.escape(Path(os.path.relpath(path, REVIEW_DIR)).as_posix(), quote=True)


def esc(value: Any) -> str:
    return html.escape(str(value), quote=True)


def prompt_tags(value: Any) -> list[str]:
    if isinstance(value, list):
        return [str(item) for item in value if str(item)]
    if value:
        return [part.strip() for part in str(value).split(",") if part.strip()]
    return []


def tag_badges(tags: list[str]) -> str:
    if not tags:
        return '<span class="muted">-</span>'
    return '<span class="tag-list">' + "".join(f'<span class="tag-badge">[{esc(tag)}]</span>' for tag in tags) + "</span>"


def timing_checks(validation: dict[str, Any]) -> dict[str, dict[str, Any]]:
    return {check["name"]: check for check in validation.get("checks", []) if "name" in check}


def browser_events(scenes: list[dict[str, Any]]) -> list[dict[str, Any]]:
    events: list[dict[str, Any]] = []
    scene_by_id = {scene["scene"]: scene for scene in scenes}
    for path in sorted(LOG_DIR.glob("*-browser-workflow.json")):
        report = load_json(path)
        scene = scene_by_id.get(report.get("scene"))
        if not scene:
            continue
        scene_start = float(scene["start_seconds"])
        for event in report.get("timeline", []):
            local = float(event["at_seconds"])
            events.append(
                {
                    "scene": report["scene"],
                    "time": scene_start + local,
                    "local": local,
                    "label": event["label"],
                    "detail": event.get("dashboard_label", ""),
                }
            )
    return sorted(events, key=lambda item: item["time"])


def audio_analyses() -> list[dict[str, Any]]:
    analyses: list[dict[str, Any]] = []
    if not AUDIO_ANALYSIS_DIR.is_dir():
        return analyses
    for path in sorted(AUDIO_ANALYSIS_DIR.glob("*.json")):
        try:
            analysis = load_json(path)
        except Exception:
            continue
        analysis["path"] = path
        analyses.append(analysis)
    return analyses


def voiceover_report() -> dict[str, Any]:
    if not VOICEOVER_REPORT.is_file():
        return {}
    try:
        return load_json(VOICEOVER_REPORT)
    except Exception:
        return {}


def command_typing_seconds(command: str, step: dict[str, Any]) -> float:
    delay_ms = float(step.get("type_delay_ms", 24))
    pre_enter = float(step.get("pre_enter_pause", 0.3))
    return max(0.45, (len(command) * delay_ms / 1000.0) + pre_enter + 0.15)


def vscode_step_seconds(step: dict[str, Any]) -> float:
    duration = 0.9
    if "edit_line" in step:
        replacement = str(step["edit_line"])
        duration += float(step.get("pause_before_edit", 0.5))
        duration += 0.2
        duration += len(replacement) * float(step.get("type_delay_ms", 55)) / 1000.0
        duration += float(step.get("pause_after_edit", 0.4))
    elif step.get("uncomment_and_set_true"):
        duration += float(step.get("pause_before_edit", 0.5))
        duration += int(step.get("delete_prefix_chars", 2)) * 0.1
        duration += float(step.get("pause_after_uncomment", 0.25))
        duration += 0.2
        duration += len("true") * float(step.get("type_delay_ms", 55)) / 1000.0
        duration += float(step.get("pause_after_edit", 0.4))
    else:
        duration += 0.35
    return max(0.5, duration)


def normalized_command_action(scene_id: str, command: str) -> tuple[str, str, str]:
    command = " ".join(command.split())
    if command == "docker ps":
        return "Terminal", "List required containers", "docker ps"
    if "localhost:8123/ping" in command:
        return "Terminal", "Check ClickHouse ping", "ClickHouse HTTP ping endpoint"
    if "localhost:9090/-/ready" in command:
        return "Terminal", "Check Prometheus readiness", "Prometheus readiness endpoint"
    if "localhost:3000/api/health" in command:
        return "Terminal", "Check Grafana health API", "Grafana health endpoint"
    if "getHealth" in command:
        return "Terminal", "Check RPC health", "Solana JSON-RPC getHealth request"
    if "jupiter-swap-clickhouse-carbon-example" in command:
        return "Terminal", "Start Jupiter example", "Run the Jupiter live example and watch logs"
    if "token-program-clickhouse-carbon-example" in command:
        return "Terminal", "Start Token Program example", "Run the Token Program live example and watch logs"
    return "Terminal", f"Run command in {scene_id}", command


def normalized_vscode_action(step: dict[str, Any]) -> tuple[str, str, str]:
    file = str(step.get("file", ""))
    line = int(step.get("line", 1))
    if file.endswith("/src/main.rs"):
        if "token-program-clickhouse" in file:
            return "VS Code", "Open Token Program main.rs", f"{file}:{line}"
        if "jupiter-swap-clickhouse" in file:
            return "VS Code", "Open Jupiter main.rs", f"{file}:{line}"
        return "VS Code", "Open example main.rs", f"{file}:{line}"
    if file == "crates/core/src/clickhouse/config.rs":
        return "VS Code", "Open ClickHouse config.rs", f"{file}:{line}"

    if "jupiter-swap-clickhouse" in file:
        example = "Jupiter"
    elif "token-program-clickhouse" in file:
        example = "Token Program"
    else:
        example = "example"

    if step.get("uncomment_and_set_true"):
        return (
            "VS Code",
            f"Enable {example} async inserts",
            f"{file}:{line} sets CLICKHOUSE_ASYNC_INSERT=true",
        )
    if "edit_line" in step:
        return "VS Code", f"Edit {example} environment file", f"{file}:{line}"
    return "VS Code", f"Show {example} environment file", f"{file}:{line}"


def add_action(
    actions: list[dict[str, Any]],
    *,
    scene_id: str,
    start: float,
    end: float,
    kind: str,
    action: str,
    detail: str = "",
    source: str,
) -> None:
    if end <= start:
        end = start + 0.35
    actions.append(
        {
            "scene": scene_id,
            "time": start,
            "end": end,
            "kind": kind,
            "action": action,
            "detail": detail,
            "source": source,
        }
    )


def event_lookup(events: list[dict[str, Any]], scene_id: str) -> dict[str, dict[str, Any]]:
    return {str(event["label"]): event for event in events if event["scene"] == scene_id}


def scene_event_time(
    events_by_label: dict[str, dict[str, Any]],
    label: str,
    scene_start: float,
    fallback_local: float,
) -> float:
    event = events_by_label.get(label)
    if event:
        return float(event["time"])
    return scene_start + fallback_local


def add_observability_actions(
    actions: list[dict[str, Any]],
    scene_id: str,
    scene_start: float,
    scene_duration: float,
    events_by_label: dict[str, dict[str, Any]],
) -> None:
    scene_end = scene_start + scene_duration
    jupiter = scene_event_time(events_by_label, "grafana:jupiter:visible", scene_start, 0.8)
    lower = scene_event_time(events_by_label, "grafana:jupiter:lower-panels-visible", scene_start, 4.4)
    token = scene_event_time(events_by_label, "grafana:token:visible", scene_start, 33.0)
    clickstack = scene_event_time(events_by_label, "clickstack:window-visible", scene_start, 38.4)
    inserts = scene_event_time(events_by_label, "clickstack:inserts-visible", scene_start, 39.4)

    add_action(
        actions,
        scene_id=scene_id,
        start=scene_start,
        end=max(lower, jupiter + 0.8),
        kind="Browser",
        action="Open Jupiter Grafana dashboard",
        detail="Jupiter dashboard loads directly",
        source="browser-log",
    )
    add_action(
        actions,
        scene_id=scene_id,
        start=lower,
        end=max(token, lower + 0.8),
        kind="Browser",
        action="Review Jupiter Grafana panels",
        detail="Application health, throughput, and processing panels",
        source="browser-log",
    )
    add_action(
        actions,
        scene_id=scene_id,
        start=token,
        end=max(clickstack, token + 0.8),
        kind="Browser",
        action="Switch to Token Program dashboard",
        detail="Confirm the same dashboard layout for the second example",
        source="browser-log",
    )
    add_action(
        actions,
        scene_id=scene_id,
        start=clickstack,
        end=max(inserts, clickstack + 0.5),
        kind="Browser",
        action="Open ClickStack dashboard",
        detail="Move from application metrics to the ClickHouse-side view",
        source="browser-log",
    )
    add_action(
        actions,
        scene_id=scene_id,
        start=inserts,
        end=scene_end,
        kind="Browser",
        action="Review ClickStack Inserts dashboard",
        detail="Rows and bytes inserted by table",
        source="browser-log",
    )


def add_clickhouse_play_actions(
    actions: list[dict[str, Any]],
    scene_id: str,
    scene_start: float,
    scene_duration: float,
    events_by_label: dict[str, dict[str, Any]],
) -> None:
    scene_end = scene_start + scene_duration
    visible = scene_event_time(events_by_label, "clickhouse-play:window-visible", scene_start, 0.0)
    menu = scene_event_time(events_by_label, "clickhouse-play:database-menu-visible", scene_start, 3.6)
    browser = scene_event_time(events_by_label, "clickhouse-play:table-browser-visible", scene_start, 7.0)
    scrolled = scene_event_time(events_by_label, "clickhouse-play:table-list-scrolled", scene_start, 12.0)
    families = scene_event_time(
        events_by_label,
        "clickhouse-play:landing-table-families-by-decoder:visible",
        scene_start,
        18.0,
    )
    largest = scene_event_time(
        events_by_label,
        "clickhouse-play:largest-populated-landing-tables:visible",
        scene_start,
        33.0,
    )
    metadata = scene_event_time(
        events_by_label,
        "clickhouse-play:largest-table-metadata:visible",
        scene_start,
        39.0,
    )
    rows = scene_event_time(
        events_by_label,
        "clickhouse-play:largest-transfer-checked-rows:visible",
        scene_start,
        42.0,
    )
    scroll_start = scene_event_time(events_by_label, "clickhouse-play:horizontal-scroll-start", scene_start, 51.0)
    scroll_complete = scene_event_time(events_by_label, "clickhouse-play:horizontal-scroll-complete", scene_start, 57.0)

    points = [
        (visible, menu, "Open ClickHouse Play", "ClickHouse Play is visible"),
        (menu, browser, "Open database menu", "Reveal default database and landing tables"),
        (browser, scrolled, "Scroll generated table list", "Show decoder-created landing tables"),
        (families, largest, "Group tables by decoder", "Account rows, instruction rows, and CPI event rows"),
        (largest, metadata, "List populated landing tables", "Order tables by rows and bytes"),
        (metadata, rows, "Open transfer table metadata", "Token Program transfer_checked table metadata"),
        (rows, scroll_start, "Open token transfer rows", "Default table query and business meaning"),
        (scroll_start, scroll_complete, "Scroll transfer columns", "Move left to right across the wide result table"),
        (scroll_complete, scene_end, "Hold final validation view", "Data is visible in ClickHouse Play"),
    ]
    for start, end, action, detail in points:
        add_action(
            actions,
            scene_id=scene_id,
            start=max(scene_start, start),
            end=min(scene_end, end),
            kind="Browser",
            action=action,
            detail=detail,
            source="workflow-derived",
        )


def runbook_actions(scenes: list[dict[str, Any]], events: list[dict[str, Any]]) -> list[dict[str, Any]]:
    runbook = load_runbook()
    scene_by_id = {scene["scene"]: scene for scene in scenes}
    actions: list[dict[str, Any]] = []

    for rb_scene in runbook.get("scenes", []):
        scene_id = rb_scene.get("id")
        scene = scene_by_id.get(scene_id)
        if not scene:
            continue
        scene_start = float(scene["start_seconds"])
        scene_duration = float(scene["duration_seconds"])
        human = rb_scene.get("human") or {}
        kind = str(human.get("type") or rb_scene.get("type") or "scene")
        steps = human.get("steps") or []
        cursor = scene_start + min(0.8, scene_duration * 0.08)
        latest_end = scene_start + max(0.2, scene_duration - min(0.8, scene_duration * 0.08))

        if kind == "terminal":
            if human.get("reuse_terminal"):
                cursor += 0.55
            for index, step in enumerate(steps):
                command = str(step.get("command", ""))
                action_seconds = command_typing_seconds(command, step)
                pause_after = max(0.0, float(step.get("pause_after", 2.0)))
                start = cursor
                end = min(latest_end, start + action_seconds + pause_after)
                kind_label, action, detail = normalized_command_action(scene_id, command)
                add_action(
                    actions,
                    scene_id=scene_id,
                    start=start,
                    end=max(start + 0.35, end),
                    kind=kind_label,
                    action=action,
                    detail=detail,
                    source="runbook-estimated",
                )
                cursor = end
        elif kind == "vscode":
            cursor = scene_start + float(human.get("initial_pause", 1.0))
            for index, step in enumerate(steps):
                action_seconds = vscode_step_seconds(step)
                pause_after = max(0.0, float(step.get("pause_after", human.get("pause_after", 3.0))))
                start = cursor
                end = min(latest_end, start + action_seconds + pause_after)
                kind_label, action, detail = normalized_vscode_action(step)
                add_action(
                    actions,
                    scene_id=scene_id,
                    start=start,
                    end=max(start + 0.35, end),
                    kind=kind_label,
                    action=action,
                    detail=detail,
                    source="runbook-estimated",
                )
                cursor = end
        elif kind == "browser":
            workflow = str(human.get("workflow") or rb_scene.get("command") or "browser workflow")
            events_by_label = event_lookup(events, scene_id)
            if workflow == "live-observability":
                add_observability_actions(actions, scene_id, scene_start, scene_duration, events_by_label)
            elif workflow == "table-inspection":
                add_clickhouse_play_actions(actions, scene_id, scene_start, scene_duration, events_by_label)
            else:
                add_action(
                    actions,
                    scene_id=scene_id,
                    start=scene_start,
                    end=scene_start + scene_duration,
                    kind="Browser",
                    action=rb_scene.get("title", workflow),
                    detail=workflow,
                    source="runbook",
                )
        else:
            add_action(
                actions,
                scene_id=scene_id,
                start=scene_start,
                end=scene_start + scene_duration,
                kind=kind.title(),
                action=rb_scene.get("title", scene_id),
                detail=rb_scene.get("script", ""),
                source="runbook",
            )

    return sorted(actions, key=lambda item: item["time"])


def find_cue(cues: list[Cue], pattern: str) -> Cue | None:
    compiled = re.compile(pattern, re.I)
    for cue in cues:
        if compiled.search(cue.text):
            return cue
    return None


def cue_review_map(cues: list[Cue], checks: dict[str, dict[str, Any]]) -> dict[int, dict[str, Any]]:
    mapping = [
        (
            r"Token Program dashboard follows",
            "scene-05:grafana-token-visible",
            "Token dashboard visible",
        ),
        (
            r"ClickStack then gives",
            "scene-05:clickstack-window-visible",
            "ClickStack visible",
        ),
        (
            r"Inserts dashboard shows",
            "scene-05:clickstack-inserts-visible",
            "ClickStack Inserts dashboard visible",
        ),
    ]
    review: dict[int, dict[str, Any]] = {}
    for pattern, check_name, label in mapping:
        cue = find_cue(cues, pattern)
        check = checks.get(check_name)
        if cue and check:
            review[cue.index] = {
                "expected": label,
                "actual_seconds": check.get("actual_seconds"),
                "expected_seconds": check.get("expected_seconds"),
                "delta_seconds": check.get("delta_seconds"),
                "status": "ok" if "reason" not in check else "check",
            }
    return review


def run_ffmpeg_frame(video: Path, seconds: float, target: Path) -> bool:
    target.parent.mkdir(parents=True, exist_ok=True)
    proc = subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-ss",
            f"{seconds:.3f}",
            "-i",
            str(video),
            "-frames:v",
            "1",
            str(target),
        ],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return proc.returncode == 0 and target.is_file()


def build_strip(name: str, start: float, end: float, samples: int = 10) -> Path | None:
    source = VIDEO if VIDEO.is_file() else RAW_VIDEO
    if not source.is_file():
        return None
    strip_dir = ASSET_DIR / name
    strip_dir.mkdir(parents=True, exist_ok=True)
    for existing in strip_dir.glob("frame-*.jpg"):
        existing.unlink()
    if samples <= 1:
        times = [start]
    else:
        step = (end - start) / (samples - 1)
        times = [start + step * index for index in range(samples)]
    frames: list[Path] = []
    for index, seconds in enumerate(times):
        frame = strip_dir / f"frame-{index:02d}.jpg"
        if run_ffmpeg_frame(source, seconds, frame):
            frames.append(frame)
    if not frames:
        return None
    output = ASSET_DIR / f"{name}.jpg"
    proc = subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-pattern_type",
            "glob",
            "-i",
            str(strip_dir / "frame-*.jpg"),
            "-vf",
            f"scale={max(220, 1800 // len(frames))}:-1,tile={len(frames)}x1",
            str(output),
        ],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        check=False,
    )
    return output if proc.returncode == 0 and output.is_file() else None


def transition_strips(checks: dict[str, dict[str, Any]]) -> list[dict[str, Any]]:
    strips: list[dict[str, Any]] = []
    for name, title in [
        ("scene-05:grafana-to-clickstack-blackout", "Grafana to ClickStack"),
        ("scene-05-to-scene-06:blackout", "ClickStack to ClickHouse Play"),
    ]:
        check = checks.get(name)
        if not check:
            continue
        expected = check.get("expected_window_seconds") or []
        if len(expected) != 2:
            continue
        start = max(0.0, float(expected[0]) - 0.55)
        end = float(expected[1]) + 0.95
        image = build_strip(re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-"), start, end)
        strips.append({"name": name, "title": title, "start": start, "end": end, "image": image})
    return strips


def status_badge(status: str) -> str:
    css = "ok" if status == "ok" else "bad"
    return f'<span class="badge {css}">{esc(status)}</span>'


def build_html() -> str:
    for path in (SRT, SUBTITLE_REPORT, TIMING_VALIDATION):
        if not path.is_file():
            raise FileNotFoundError(path)
    report = load_json(SUBTITLE_REPORT)
    validation = load_json(TIMING_VALIDATION)
    scenes = report["scenes"]
    cues = parse_srt(SRT, scenes)
    checks = timing_checks(validation)
    events = browser_events(scenes)
    actions = runbook_actions(scenes, events)
    audio_reports = audio_analyses()
    voiceover = voiceover_report()
    voiceover_placements = voiceover.get("placements", [])
    cue_reviews = cue_review_map(cues, checks)
    total = float(report["total_duration_seconds"])
    strips = transition_strips(checks)
    failures = validation.get("failures", [])

    scene_rows = []
    for scene in scenes:
        tail = checks.get(f"{scene['scene']}:tail-silence", {}).get("tail_seconds", "")
        scene_rows.append(
            f"""
            <tr>
              <td>{esc(scene["scene"])}</td>
              <td>{fmt_time(float(scene["start_seconds"]))}</td>
              <td>{fmt_time(float(scene["duration_seconds"]))}</td>
              <td>{esc(scene.get("caption_words_per_minute", ""))}</td>
              <td>{esc(tail)}</td>
              <td>{status_badge("ok" if scene.get("natural_pacing_fits_scene") else "check")}</td>
            </tr>
            """
        )

    cue_rows = []
    for cue in cues:
        review = cue_reviews.get(cue.index, {})
        actual = review.get("actual_seconds")
        delta = review.get("delta_seconds")
        event_text = ""
        if review:
            event_text = (
                f'{esc(review["expected"])}<br>'
                f'<span class="muted">actual scene-local {esc(actual)}s, delta {esc(delta)}s</span>'
            )
        cue_rows.append(
            f"""
            <tr data-seek="{cue.start:.3f}" class="{'key' if review else ''}">
              <td>{cue.index}</td>
              <td>{fmt_time(cue.start)}</td>
              <td>{fmt_time(cue.end - cue.start)}</td>
              <td>{esc(cue.scene)}</td>
              <td>{esc(cue.text)}</td>
              <td>{event_text}</td>
              <td>{status_badge(review.get("status", "ok")) if review else ""}</td>
            </tr>
            """
        )

    transition_rows = []
    for name, check in checks.items():
        if "blackout" not in name:
            continue
        intervals = check.get("matched_intervals", [])
        interval_text = "<br>".join(
            f'{fmt_time(float(item["start_seconds"]))} - {fmt_time(float(item["end_seconds"]))} ({item["duration_seconds"]}s)'
            for item in intervals
        )
        expected = check.get("expected_window_seconds", [])
        expected_text = f"{fmt_time(float(expected[0]))} - {fmt_time(float(expected[1]))}" if len(expected) == 2 else ""
        transition_rows.append(
            f"""
            <tr>
              <td>{esc(name)}</td>
              <td>{expected_text}</td>
              <td>{interval_text}</td>
              <td>{status_badge("ok" if intervals else "check")}</td>
            </tr>
            """
        )

    timeline_center = 310
    scene_bands = []
    for index, scene in enumerate(scenes):
        start = float(scene["start_seconds"])
        duration = float(scene["duration_seconds"])
        scene_bands.append(
            f'<button class="scene-band shade-{index % 2}" data-seek="{start:.3f}" style="left:{pct(start, total)};width:{pct(duration, total)}" title="{esc(scene["scene"])}">'
            f'<span>{esc(scene["scene"].replace("scene-", ""))}</span></button>'
        )

    time_ticks = []
    tick = 0
    while tick <= total + 0.001:
        time_ticks.append(
            f'<button class="time-tick" data-seek="{tick:.3f}" style="left:{pct(tick, total)}" title="{fmt_time(tick)}">'
            f'<span>{fmt_time(tick)}</span></button>'
        )
        tick += 15

    subtitle_spans = []
    subtitle_pins = []
    for cue in cues:
        lane = (cue.index - 1) % 7
        top = 40 + lane * 34
        stem = max(24, timeline_center - top - 30)
        title = f"{cue.index}: {cue.text}"
        subtitle_spans.append(
            f'<button class="subtitle-span" data-seek="{cue.start:.3f}" style="left:{pct(cue.start, total)};width:{pct(cue.end - cue.start, total)}" title="{esc(title)}"></button>'
        )
        subtitle_pins.append(
            f'<button class="timeline-pin subtitle-pin" data-seek="{cue.start:.3f}" style="left:{pct(cue.start, total)};top:{top}px;--stem:{stem}px" title="{esc(title)}">'
            f'<span class="pin-label">{esc(cue.text)}</span></button>'
        )

    action_spans = []
    action_pins = []
    for index, action in enumerate(actions):
        start = float(action["time"])
        duration = max(0.3, float(action["end"]) - start)
        lane = index % 7
        top = timeline_center + 26 + lane * 39
        stem = max(18, top - timeline_center)
        title = f'{action["scene"]}: {action["action"]} [{action["source"]}]'
        action_spans.append(
            f'<button class="action-span" data-seek="{start:.3f}" style="left:{pct(start, total)};width:{pct(duration, total)}" title="{esc(title)}"></button>'
        )
        action_pins.append(
            f'<button class="timeline-pin action-pin" data-seek="{start:.3f}" style="left:{pct(start, total)};top:{top}px;--stem:{stem}px" title="{esc(title)}">'
            f'<span class="pin-label">{esc(action["action"])}</span></button>'
        )

    audio_spans = []
    audio_pins = []
    audio_rows = []
    audio_cards = []
    audio_index = 0
    for analysis in audio_reports:
        provider = analysis.get("provider", "audio")
        scene = analysis.get("scene", "")
        analysis_tags = prompt_tags(analysis.get("prompt_tags", []))
        audio_path = ROOT / str(analysis.get("audio", ""))
        analysis_path = analysis.get("path")
        if audio_path.is_file():
            audio_cards.append(
                f"""
                <article class="audio-card">
                  <h3>{esc(provider)} {esc(scene)}</h3>
                  <audio controls preload="metadata" src="{rel(audio_path)}"></audio>
                  <div class="file-list compact">
                    <a href="{rel(audio_path)}">audio</a>
                    {f'<a href="{rel(analysis_path)}">analysis JSON</a>' if isinstance(analysis_path, Path) else ''}
                  </div>
                  <div class="prompt-tags">{tag_badges(analysis_tags)}</div>
                  <p class="muted">{esc(analysis.get("method", ""))}</p>
                </article>
                """
            )
        for utterance in analysis.get("utterances", []):
            audio_index += 1
            start = float(utterance["timeline_start_seconds"])
            end = float(utterance["timeline_end_seconds"])
            duration = max(0.2, end - start)
            lane = (audio_index - 1) % 4
            top = timeline_center + 318 + lane * 48
            stem = max(18, top - timeline_center)
            text = str(utterance.get("text", ""))
            tags = prompt_tags(utterance.get("prompt_tags") or utterance.get("prompt_tag"))
            tag_label = " ".join(f"[{tag}]" for tag in tags)
            timeline_label = f"{tag_label} {text}".strip()
            title = f'{provider} {scene}: {timeline_label}'
            audio_spans.append(
                f'<button class="audio-span" data-seek="{start:.3f}" style="left:{pct(start, total)};width:{pct(duration, total)}" title="{esc(title)}"></button>'
            )
            audio_pins.append(
                f'<button class="timeline-pin audio-pin" data-seek="{start:.3f}" style="left:{pct(start, total)};top:{top}px;--stem:{stem}px" title="{esc(title)}">'
                f'<span class="pin-label">{esc(timeline_label)}</span></button>'
            )
            audio_rows.append(
                f"""
                <tr data-seek="{start:.3f}">
                  <td>{esc(provider)}</td>
                  <td>{esc(scene)}</td>
                  <td>{fmt_time(start)}</td>
                  <td>{fmt_time(duration)}</td>
                  <td>{fmt_time(float(utterance["start_seconds"]))}</td>
                  <td>{tag_badges(tags)}</td>
                  <td>{esc(text)}</td>
                  <td>{esc(utterance.get("text_source", ""))}</td>
                </tr>
                """
            )

    voiceover_spans = []
    voiceover_rows = []
    for index, placement in enumerate(voiceover_placements, start=1):
        start = float(placement["target_timeline_start_seconds"])
        duration = max(0.1, float(placement.get("clip_duration_seconds", 0.0)))
        tags = prompt_tags(placement.get("prompt_tags") or placement.get("prompt_tag"))
        tag_label = " ".join(f"[{tag}]" for tag in tags)
        text = str(placement.get("text", ""))
        title = f'{placement.get("scene", "")} #{placement.get("utterance_index", index)}: {tag_label} {text}'.strip()
        target_detail = placement.get("target_event") or placement.get("target_action") or placement.get("target_source", "")
        voiceover_spans.append(
            f'<button class="voiceover-span" data-seek="{start:.3f}" style="left:{pct(start, total)};width:{pct(duration, total)}" title="{esc(title)}"></button>'
        )
        voiceover_rows.append(
            f"""
            <tr data-seek="{start:.3f}">
              <td>{esc(placement.get("scene", ""))}</td>
              <td>{esc(placement.get("utterance_index", index))}</td>
              <td>{fmt_time(start)}</td>
              <td>{fmt_time(duration)}</td>
              <td>{fmt_time(float(placement.get("source_cut_start_seconds", 0.0)))} - {fmt_time(float(placement.get("source_cut_end_seconds", 0.0)))}</td>
              <td>{esc(placement.get("target_source", ""))}</td>
              <td>{esc(target_detail)}</td>
              <td>{tag_badges(tags)}</td>
              <td>{esc(text)}</td>
            </tr>
            """
        )

    blackout_spans = []
    for check in checks.values():
        if not str(check.get("name", "")).startswith("black-interval-"):
            continue
        start = float(check["start_seconds"])
        duration = float(check["duration_seconds"])
        blackout_spans.append(
            f'<button class="blackout-span" data-seek="{start:.3f}" style="left:{pct(start, total)};width:{pct(duration, total)}" title="blackout {fmt_time(start)} duration {duration}s">'
            f'<span>blackout</span></button>'
        )

    action_rows = []
    for action in actions:
        start = float(action["time"])
        end = float(action["end"])
        action_rows.append(
            f"""
            <tr data-seek="{start:.3f}">
              <td>{fmt_time(start)}</td>
              <td>{fmt_time(end - start)}</td>
              <td>{esc(action["scene"])}</td>
              <td>{esc(action["kind"])}</td>
              <td>{esc(action["action"])}</td>
              <td>{esc(action["detail"])}</td>
              <td>{esc(action["source"])}</td>
            </tr>
            """
        )

    strip_cards = []
    for strip in strips:
        if not strip.get("image"):
            continue
        image = strip["image"]
        strip_cards.append(
            f"""
            <article class="strip-card">
              <h3>{esc(strip["title"])}</h3>
              <button class="linklike" data-seek="{float(strip["start"]):.3f}">{fmt_time(float(strip["start"]))} - {fmt_time(float(strip["end"]))}</button>
              <img src="{rel(image)}" alt="{esc(strip["title"])} frame strip">
            </article>
            """
        )

    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Subtitle Alignment Timeline</title>
<style>
:root {{
  color-scheme: dark;
  --bg: #080b0f;
  --panel: #111820;
  --panel2: #18222d;
  --line: #2b3a46;
  --text: #e8edf2;
  --muted: #9aa8b4;
  --accent: #faff69;
  --cyan: #65d4ff;
  --green: #63e6be;
  --orange: #ffb266;
  --audio: #75e4ff;
  --voice: #d49bff;
  --red: #ff7a90;
}}
* {{ box-sizing: border-box; }}
body {{
  margin: 0;
  background: var(--bg);
  color: var(--text);
  font: 14px/1.45 system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}}
header {{
  position: sticky;
  top: 0;
  z-index: 10;
  background: rgba(8, 11, 15, 0.94);
  border-bottom: 1px solid var(--line);
  padding: 14px 20px;
}}
h1 {{ margin: 0 0 6px; font-size: 20px; }}
h2 {{ margin: 28px 0 12px; font-size: 17px; }}
h3 {{ margin: 0 0 8px; font-size: 14px; }}
a {{ color: var(--cyan); }}
main {{ max-width: 1480px; margin: 0 auto; padding: 18px 20px 48px; }}
.top-grid {{ display: grid; grid-template-columns: minmax(420px, 820px) 1fr; gap: 18px; align-items: start; }}
video {{ width: 100%; background: #000; border: 1px solid var(--line); border-radius: 6px; }}
.panel {{ background: var(--panel); border: 1px solid var(--line); border-radius: 6px; padding: 14px; }}
.facts {{ display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }}
.fact {{ background: var(--panel2); border: 1px solid var(--line); border-radius: 4px; padding: 10px; }}
.fact b {{ display:block; font-size: 12px; color: var(--muted); font-weight: 500; }}
.fact span {{ font-size: 20px; }}
.badge {{ display: inline-flex; align-items: center; height: 22px; padding: 0 8px; border-radius: 99px; font-size: 12px; font-weight: 600; }}
.badge.ok {{ background: rgba(99, 230, 190, .14); color: var(--green); border: 1px solid rgba(99, 230, 190, .4); }}
.badge.bad, .badge.check {{ background: rgba(255, 122, 144, .14); color: var(--red); border: 1px solid rgba(255, 122, 144, .45); }}
.muted {{ color: var(--muted); font-size: 12px; }}
.timeline {{ position: relative; height: 836px; border: 1px solid var(--line); border-radius: 6px; overflow-x: auto; overflow-y: hidden; background: #0b1015; }}
.timeline-inner {{ position: relative; min-width: 5400px; height: 100%; }}
.center-line {{ position: absolute; left: 0; right: 0; top: 310px; height: 1px; background: #5b6872; z-index: 2; }}
.track-title {{ position: absolute; left: 12px; z-index: 8; color: var(--muted); font-size: 12px; font-weight: 650; letter-spacing: .02em; }}
.track-title.upper {{ top: 10px; }}
.track-title.lower {{ top: 612px; }}
.track-title.audio {{ top: 802px; color: var(--audio); }}
.scene-band, .subtitle-span, .action-span, .audio-span, .voiceover-span, .time-tick, .timeline-pin, .blackout-span {{
  position: absolute;
  border: 0;
  padding: 0;
  cursor: pointer;
}}
.scene-band {{ top: 0; bottom: 0; background: rgba(101, 212, 255, .045); border-left: 1px solid rgba(101, 212, 255, .28); color: var(--muted); text-align: left; z-index: 0; }}
.scene-band.shade-1 {{ background: rgba(255, 255, 255, .025); }}
.scene-band span {{ position: absolute; top: 292px; left: 8px; max-width: calc(100% - 12px); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; }}
.time-tick {{ top: 304px; width: 1px; height: 14px; background: #73808a; z-index: 3; overflow: visible; }}
.time-tick span {{ position: absolute; top: 18px; left: -18px; color: var(--muted); font-size: 10px; white-space: nowrap; }}
.subtitle-span {{ top: 295px; height: 5px; background: rgba(250, 255, 105, .74); border-radius: 2px; min-width: 2px; z-index: 4; }}
.action-span {{ top: 320px; height: 7px; background: rgba(255, 178, 102, .72); border-radius: 2px; min-width: 4px; z-index: 4; }}
.audio-span {{ top: 334px; height: 6px; background: rgba(117, 228, 255, .82); border-radius: 2px; min-width: 3px; z-index: 5; }}
.voiceover-span {{ top: 344px; height: 8px; background: rgba(212, 155, 255, .78); border-radius: 2px; min-width: 4px; z-index: 5; }}
.blackout-span {{ top: 278px; height: 22px; min-width: 10px; background: #000; border: 1px solid #777; color: #c7cdd3; border-radius: 3px; z-index: 6; }}
.blackout-span span {{ display: block; padding: 2px 5px; font-size: 10px; white-space: nowrap; }}
.timeline-pin {{ width: 270px; background: transparent; text-align: left; z-index: 7; transform: translateX(-1px); }}
.timeline-pin::before, .timeline-pin::after {{ content: ""; position: absolute; left: 0; width: 1px; pointer-events: none; }}
.subtitle-pin {{ color: var(--accent); }}
.subtitle-pin::after {{ top: 30px; height: var(--stem); background: var(--accent); }}
.action-pin {{ color: var(--orange); }}
.action-pin::before {{ top: calc(-1 * var(--stem)); height: var(--stem); background: var(--orange); }}
.audio-pin {{ color: var(--audio); }}
.audio-pin::before {{ top: calc(-1 * var(--stem)); height: var(--stem); background: var(--audio); }}
.pin-label {{ display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; max-width: 250px; min-height: 26px; padding: 4px 6px; border-radius: 4px; background: rgba(8, 11, 15, .86); border: 1px solid rgba(255, 255, 255, .12); font-size: 11px; line-height: 1.2; }}
.subtitle-pin .pin-label {{ border-color: rgba(250, 255, 105, .34); }}
.action-pin .pin-label {{ border-color: rgba(255, 178, 102, .34); }}
.audio-pin .pin-label {{ border-color: rgba(117, 228, 255, .46); }}
table {{ width: 100%; border-collapse: collapse; }}
th, td {{ border-bottom: 1px solid var(--line); padding: 8px 9px; text-align: left; vertical-align: top; }}
th {{ color: var(--muted); font-size: 12px; font-weight: 600; background: #0d131a; position: sticky; top: 62px; z-index: 4; }}
tr[data-seek] {{ cursor: pointer; }}
tr[data-seek]:hover td {{ background: rgba(101, 212, 255, .08); }}
tr.key td {{ background: rgba(250, 255, 105, .045); }}
.strips {{ display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; }}
.strip-card img {{ display:block; width: 100%; border: 1px solid var(--line); border-radius: 5px; }}
.linklike {{ color: var(--cyan); background: transparent; border: 0; padding: 0; cursor: pointer; font: inherit; margin-bottom: 8px; }}
.file-list {{ display: flex; flex-wrap: wrap; gap: 10px; margin-top: 8px; }}
.file-list a {{ background: var(--panel2); border: 1px solid var(--line); border-radius: 4px; padding: 5px 8px; text-decoration: none; }}
.file-list.compact {{ margin-top: 6px; gap: 6px; }}
.audio-cards {{ display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 14px; margin-bottom: 14px; }}
.audio-card audio {{ width: 100%; }}
.prompt-tags {{ margin: 8px 0 6px; }}
.tag-list {{ display: flex; flex-wrap: wrap; gap: 5px; align-items: center; }}
.tag-badge {{ display: inline-flex; align-items: center; min-height: 20px; padding: 2px 6px; border-radius: 4px; border: 1px solid rgba(117, 228, 255, .42); background: rgba(117, 228, 255, .1); color: var(--audio); font-size: 11px; font-weight: 650; white-space: nowrap; }}
@media (max-width: 980px) {{
  .top-grid, .strips, .audio-cards {{ grid-template-columns: 1fr; }}
}}
</style>
</head>
<body>
<header>
  <h1>Subtitle Alignment Timeline</h1>
  <div class="muted">Generated from the current review video, SRT, browser workflow logs, and timing validation report.</div>
</header>
<main>
  <section class="top-grid">
    <div>
      <video id="review-video" controls preload="metadata">
        <source src="{rel(VIDEO if VIDEO.is_file() else RAW_VIDEO)}" type="video/mp4">
        <track src="{rel(VTT)}" kind="subtitles" srclang="en" label="English" default>
      </video>
      <div class="file-list">
        <a href="{rel(VIDEO if VIDEO.is_file() else RAW_VIDEO)}">video</a>
        <a href="{rel(SRT)}">SRT</a>
        <a href="{rel(VTT)}">VTT</a>
        <a href="{rel(TIMING_VALIDATION)}">timing JSON</a>
        <a href="{rel(SUBTITLE_REPORT)}">subtitle report</a>
        <a href="{rel(REVIEW_DIR / "runbook.yaml")}">review runbook</a>
      </div>
    </div>
    <aside class="panel">
      <div class="facts">
        <div class="fact"><b>Timing Validation</b><span>{esc(validation.get("status", "unknown"))}</span></div>
        <div class="fact"><b>Total Duration</b><span>{fmt_time(total)}</span></div>
        <div class="fact"><b>Subtitle Cues</b><span>{len(cues)}</span></div>
        <div class="fact"><b>Runbook Actions</b><span>{len(actions)}</span></div>
        <div class="fact"><b>Audio Utterances</b><span>{audio_index}</span></div>
        <div class="fact"><b>Voiceover Placements</b><span>{len(voiceover_placements)}</span></div>
        <div class="fact"><b>Failures</b><span>{len(failures)}</span></div>
      </div>
      <p class="muted">Click any cue, action, scene band, table row, or frame strip timestamp to seek the video.</p>
    </aside>
  </section>

  <h2>Timeline</h2>
  <section class="timeline" aria-label="subtitle alignment timeline">
    <div class="timeline-inner">
      <div class="track-title upper">Subtitles and blackouts</div>
      <div class="track-title lower">Actions</div>
      <div class="track-title audio">Audio analysis</div>
      <div class="center-line"></div>
      {''.join(scene_bands)}
      {''.join(time_ticks)}
      {''.join(subtitle_spans)}
      {''.join(blackout_spans)}
      {''.join(action_spans)}
      {''.join(audio_spans)}
      {''.join(voiceover_spans)}
      {''.join(subtitle_pins)}
      {''.join(action_pins)}
      {''.join(audio_pins)}
    </div>
  </section>

  <h2>Transition Strips</h2>
  <section class="strips">
    {''.join(strip_cards)}
  </section>

  <h2>Scene Summary</h2>
  <div class="panel">
    <table>
      <thead><tr><th>Scene</th><th>Start</th><th>Duration</th><th>WPM</th><th>Tail Silence</th><th>Status</th></tr></thead>
      <tbody>{''.join(scene_rows)}</tbody>
    </table>
  </div>

  <h2>Action Timeline</h2>
  <div class="panel">
    <p class="muted">Terminal and VS Code timing is estimated from runbook order, pauses, and visible typing/edit durations. Browser actions use exact workflow-log timestamps where available and deterministic workflow milestones elsewhere.</p>
    <table>
      <thead><tr><th>Start</th><th>Duration</th><th>Scene</th><th>Kind</th><th>Action</th><th>Detail</th><th>Source</th></tr></thead>
      <tbody>{''.join(action_rows)}</tbody>
    </table>
  </div>

  <h2>Audio Analysis</h2>
  <div class="panel">
    <p class="muted">Speech timings are detected from the WAV with FFmpeg silencedetect and mapped to the known generated transcript. This is forced alignment, not independent ASR.</p>
    <div class="audio-cards">{''.join(audio_cards)}</div>
    <table>
      <thead><tr><th>Provider</th><th>Scene</th><th>Timeline Start</th><th>Duration</th><th>Audio Local Start</th><th>Prompt Tag</th><th>Detected/Aligned Speech</th><th>Text Source</th></tr></thead>
      <tbody>{''.join(audio_rows)}</tbody>
    </table>
  </div>

  <h2>Voiceover Placement</h2>
  <div class="panel">
    <p class="muted">Purple timeline spans show where analyzed narration clips are placed on the final video timeline. Placement cuts use silence/speech boundaries from the source scene audio.</p>
    <div class="file-list compact">
      {f'<a href="{rel(VOICEOVER_REPORT)}">placement report</a>' if VOICEOVER_REPORT.is_file() else ''}
      {f'<a href="{rel(REVIEW_DIR / "audio/voiceover-timeline.wav")}">voiceover WAV</a>' if (REVIEW_DIR / "audio/voiceover-timeline.wav").is_file() else ''}
      {f'<a href="{rel(REVIEW_DIR / "videos/clickhouse-sink-tutorial-human-voiceover.mp4")}">voiceover video</a>' if (REVIEW_DIR / "videos/clickhouse-sink-tutorial-human-voiceover.mp4").is_file() else ''}
    </div>
    <table>
      <thead><tr><th>Scene</th><th>#</th><th>Target Start</th><th>Duration</th><th>Source Cut</th><th>Target Source</th><th>Target Detail</th><th>Prompt Tag</th><th>Text</th></tr></thead>
      <tbody>{''.join(voiceover_rows)}</tbody>
    </table>
  </div>

  <h2>Transitions</h2>
  <div class="panel">
    <table>
      <thead><tr><th>Check</th><th>Expected Window</th><th>Detected Interval</th><th>Status</th></tr></thead>
      <tbody>{''.join(transition_rows)}</tbody>
    </table>
  </div>

  <h2>Subtitle Cues</h2>
  <div class="panel">
    <table>
      <thead><tr><th>#</th><th>Start</th><th>Duration</th><th>Scene</th><th>Subtitle</th><th>Aligned Screen Event</th><th>Status</th></tr></thead>
      <tbody>{''.join(cue_rows)}</tbody>
    </table>
  </div>
</main>
<script>
const video = document.getElementById('review-video');
function seek(seconds) {{
  video.currentTime = Number(seconds);
  video.play().catch(() => {{}});
  video.scrollIntoView({{ behavior: 'smooth', block: 'start' }});
}}
document.querySelectorAll('[data-seek]').forEach((element) => {{
  element.addEventListener('click', () => seek(element.dataset.seek));
}});
</script>
</body>
</html>
"""


def main() -> int:
    ASSET_DIR.mkdir(parents=True, exist_ok=True)
    OUTPUT.write_text(build_html())
    print(OUTPUT)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
