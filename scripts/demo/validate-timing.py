#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
REVIEW_DIR = ROOT / "demo-artifacts/review-human"
LOG_DIR = REVIEW_DIR / "logs"
SUBTITLE_DIR = REVIEW_DIR / "subtitles"
VIDEO = REVIEW_DIR / "videos/clickhouse-sink-tutorial-human-no-audio.mp4"
SRT = SUBTITLE_DIR / "clickhouse-sink-tutorial-human-no-audio.srt"
SUBTITLE_REPORT = SUBTITLE_DIR / "subtitle-report.json"
OUTPUT = LOG_DIR / "timing-validation.json"

MAX_SCENE_TAIL_SECONDS = 2.5
MAX_BLACK_INTERVAL_SECONDS = 1.25
MAX_BLACK_CUE_OVERLAP_SECONDS = 0.25
MAX_LOG_VIDEO_DRIFT_SECONDS = 30 * 60


@dataclass(frozen=True)
class Cue:
    start: float
    end: float
    text: str


def parse_srt_time(value: str) -> float:
    hours, minutes, rest = value.split(":")
    seconds, millis = rest.split(",")
    return int(hours) * 3600 + int(minutes) * 60 + int(seconds) + int(millis) / 1000


def parse_srt(path: Path) -> list[Cue]:
    cues: list[Cue] = []
    for block in path.read_text().strip().split("\n\n"):
        lines = [line.strip() for line in block.splitlines() if line.strip()]
        if len(lines) < 3 or "-->" not in lines[1]:
            continue
        raw_start, raw_end = [part.strip() for part in lines[1].split("-->", 1)]
        cues.append(Cue(parse_srt_time(raw_start), parse_srt_time(raw_end), " ".join(lines[2:])))
    return cues


def run_blackdetect(video: Path) -> list[dict[str, float]]:
    proc = subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-i",
            str(video),
            "-vf",
            "blackdetect=d=0.35:pic_th=0.98:pix_th=0.05",
            "-an",
            "-f",
            "null",
            "-",
        ],
        cwd=ROOT,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError("ffmpeg blackdetect failed")
    intervals: list[dict[str, float]] = []
    for match in re.finditer(
        r"black_start:(?P<start>[0-9.]+)\s+black_end:(?P<end>[0-9.]+)\s+black_duration:(?P<duration>[0-9.]+)",
        proc.stderr,
    ):
        intervals.append({key: float(match.group(key)) for key in ("start", "end", "duration")})
    return intervals


def overlap_seconds(a_start: float, a_end: float, b_start: float, b_end: float) -> float:
    return max(0.0, min(a_end, b_end) - max(a_start, b_start))


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def scene_for_time(scenes: list[dict[str, Any]], seconds: float) -> dict[str, Any] | None:
    for scene in scenes:
        start = float(scene["start_seconds"])
        end = start + float(scene["duration_seconds"])
        if start <= seconds < end:
            return scene
    return None


def cue_matching(cues: list[Cue], pattern: str) -> Cue | None:
    compiled = re.compile(pattern, re.I)
    for cue in cues:
        if compiled.search(cue.text):
            return cue
    return None


def scene_local(scene: dict[str, Any], seconds: float) -> float:
    return seconds - float(scene["start_seconds"])


def timeline_event(path: Path, label: str) -> float | None:
    if not path.is_file():
        return None
    report = load_json(path)
    for event in report.get("timeline", []):
        if event.get("label") == label:
            return float(event["at_seconds"])
    return None


def add_timeline_freshness_check(checks: list[dict[str, Any]], failures: list[dict[str, Any]], path: Path) -> None:
    check = {"name": f"{path.name}:freshness", "path": str(path.relative_to(ROOT))}
    checks.append(check)
    if not path.is_file():
        failures.append({**check, "reason": "timeline log is missing"})
        return
    video_mtime = VIDEO.stat().st_mtime
    log_mtime = path.stat().st_mtime
    delta = log_mtime - video_mtime
    check["log_minus_video_seconds"] = round(delta, 3)
    if delta > 5:
        failures.append({**check, "reason": "timeline log is newer than the final video"})
    elif abs(delta) > MAX_LOG_VIDEO_DRIFT_SECONDS:
        failures.append({**check, "reason": "timeline log and final video appear to come from different runs"})


def add_timing_check(
    *,
    checks: list[dict[str, Any]],
    failures: list[dict[str, Any]],
    name: str,
    actual: float | None,
    expected: float,
    min_delta: float,
    max_delta: float,
) -> None:
    check = {
        "name": name,
        "actual_seconds": actual,
        "expected_seconds": round(expected, 3),
        "allowed_delta_seconds": [min_delta, max_delta],
    }
    checks.append(check)
    if actual is None:
        failures.append({**check, "reason": "missing timeline event"})
        return
    delta = actual - expected
    check["delta_seconds"] = round(delta, 3)
    if delta < min_delta or delta > max_delta:
        failures.append({**check, "reason": "timeline event outside subtitle cue window"})


def add_black_transition_check(
    *,
    checks: list[dict[str, Any]],
    failures: list[dict[str, Any]],
    name: str,
    intervals: list[dict[str, float]],
    earliest: float,
    latest: float,
) -> None:
    candidates = [
        interval
        for interval in intervals
        if interval["duration"] <= MAX_BLACK_INTERVAL_SECONDS
        and overlap_seconds(interval["start"], interval["end"], earliest, latest) > 0
    ]
    check = {
        "name": name,
        "expected_window_seconds": [round(earliest, 3), round(latest, 3)],
        "matched_intervals": [
            {
                "start_seconds": round(interval["start"], 3),
                "end_seconds": round(interval["end"], 3),
                "duration_seconds": round(interval["duration"], 3),
            }
            for interval in candidates
        ],
    }
    checks.append(check)
    if not candidates:
        failures.append({**check, "reason": "expected smooth blackout transition was not detected"})


def matching_transition_windows(
    interval: dict[str, float],
    windows: list[dict[str, Any]],
) -> list[dict[str, Any]]:
    return [
        window
        for window in windows
        if overlap_seconds(interval["start"], interval["end"], window["start"], window["end"]) > 0
    ]


def validate() -> dict[str, Any]:
    for required in (VIDEO, SRT, SUBTITLE_REPORT):
        if not required.is_file():
            raise FileNotFoundError(required)

    cues = parse_srt(SRT)
    report = load_json(SUBTITLE_REPORT)
    scenes = report["scenes"]
    checks: list[dict[str, Any]] = []
    failures: list[dict[str, Any]] = []

    for scene in scenes:
        start = float(scene["start_seconds"])
        end = start + float(scene["duration_seconds"])
        scene_cues = [cue for cue in cues if start <= cue.start < end]
        if not scene_cues:
            failures.append({"name": f"{scene['scene']}:subtitle-cues", "reason": "scene has no subtitle cues"})
            continue
        tail = end - scene_cues[-1].end
        check = {"name": f"{scene['scene']}:tail-silence", "tail_seconds": round(tail, 3)}
        checks.append(check)
        if tail > MAX_SCENE_TAIL_SECONDS:
            failures.append({**check, "reason": "scene continues too long after final subtitle cue"})

    scene5 = next((scene for scene in scenes if scene["scene"] == "scene-05-observability"), None)
    scene6 = next((scene for scene in scenes if scene["scene"] == "scene-06-clickhouse-play"), None)
    token_cue = cue_matching(cues, r"Token Program dashboard follows")
    clickstack_cue = cue_matching(cues, r"ClickStack then gives")
    inserts_cue = cue_matching(cues, r"Inserts dashboard shows")
    scroll_cue = cue_matching(cues, r"As we scroll across the columns")
    token_movement_cue = cue_matching(cues, r"decoded token movement activity")
    expected_transition_windows: list[dict[str, Any]] = []
    for scene in scenes:
        start = float(scene["start_seconds"])
        expected_transition_windows.append(
            {
                "name": f"{scene['scene']}:scene-start",
                "start": max(0.0, start - 0.55),
                "end": start + 1.20,
            }
        )
    if token_cue and clickstack_cue:
        expected_transition_windows.append(
            {
                "name": "scene-05:grafana-to-clickstack",
                "start": token_cue.end - 0.25,
                "end": clickstack_cue.start + 0.25,
            }
        )

    black_intervals = run_blackdetect(VIDEO)
    for index, interval in enumerate(black_intervals, start=1):
        matched_windows = matching_transition_windows(interval, expected_transition_windows)
        check = {
            "name": f"black-interval-{index}",
            "start_seconds": round(interval["start"], 3),
            "end_seconds": round(interval["end"], 3),
            "duration_seconds": round(interval["duration"], 3),
            "expected_transition_windows": [window["name"] for window in matched_windows],
        }
        checks.append(check)
        if interval["duration"] > MAX_BLACK_INTERVAL_SECONDS:
            failures.append({**check, "reason": "black interval is longer than the transition budget"})
        for cue in cues:
            if not matched_windows:
                continue
            overlap = overlap_seconds(interval["start"], interval["end"], cue.start, cue.end)
            if overlap > MAX_BLACK_CUE_OVERLAP_SECONDS:
                scene = scene_for_time(scenes, cue.start)
                failures.append(
                    {
                        **check,
                        "reason": "black transition overlaps active subtitle cue",
                        "overlap_seconds": round(overlap, 3),
                        "cue_text": cue.text,
                        "scene": scene["scene"] if scene else None,
                    }
                )

    scene5_log = LOG_DIR / "scene-05-observability-browser-workflow.json"
    scene6_log = LOG_DIR / "scene-06-clickhouse-play-browser-workflow.json"
    add_timeline_freshness_check(checks, failures, scene5_log)
    add_timeline_freshness_check(checks, failures, scene6_log)
    if scene5:
        if token_cue:
            add_timing_check(
                checks=checks,
                failures=failures,
                name="scene-05:grafana-token-visible",
                actual=timeline_event(scene5_log, "grafana:token:visible"),
                expected=scene_local(scene5, token_cue.start),
                min_delta=-0.45,
                max_delta=1.20,
            )
        if clickstack_cue:
            add_timing_check(
                checks=checks,
                failures=failures,
                name="scene-05:clickstack-window-visible",
                actual=timeline_event(scene5_log, "clickstack:window-visible"),
                expected=scene_local(scene5, clickstack_cue.start),
                min_delta=-0.85,
                max_delta=1.00,
            )
        if inserts_cue:
            add_timing_check(
                checks=checks,
                failures=failures,
                name="scene-05:clickstack-inserts-visible",
                actual=timeline_event(scene5_log, "clickstack:inserts-visible"),
                expected=scene_local(scene5, inserts_cue.start),
                min_delta=-3.50,
                max_delta=0.75,
            )
        if token_cue and clickstack_cue:
            add_black_transition_check(
                checks=checks,
                failures=failures,
                name="scene-05:grafana-to-clickstack-blackout",
                intervals=black_intervals,
                earliest=token_cue.end - 0.25,
                latest=clickstack_cue.start + 0.25,
            )
    if scene5 and scene6:
        scene6_start = float(scene6["start_seconds"])
        add_black_transition_check(
            checks=checks,
            failures=failures,
            name="scene-05-to-scene-06:blackout",
            intervals=black_intervals,
            earliest=scene6_start - 0.35,
            latest=scene6_start + 0.85,
        )
    if scene6:
        if scroll_cue:
            add_timing_check(
                checks=checks,
                failures=failures,
                name="scene-06:horizontal-scroll-start",
                actual=timeline_event(scene6_log, "clickhouse-play:horizontal-scroll-start"),
                expected=scene_local(scene6, scroll_cue.start),
                min_delta=-0.35,
                max_delta=0.45,
            )
        if token_movement_cue:
            add_timing_check(
                checks=checks,
                failures=failures,
                name="scene-06:horizontal-scroll-complete",
                actual=timeline_event(scene6_log, "clickhouse-play:horizontal-scroll-complete"),
                expected=scene_local(scene6, token_movement_cue.end),
                min_delta=-0.30,
                max_delta=0.35,
            )

    return {
        "status": "failed" if failures else "ok",
        "thresholds": {
            "max_scene_tail_seconds": MAX_SCENE_TAIL_SECONDS,
            "max_black_interval_seconds": MAX_BLACK_INTERVAL_SECONDS,
            "max_black_cue_overlap_seconds": MAX_BLACK_CUE_OVERLAP_SECONDS,
        },
        "checks": checks,
        "failures": failures,
    }


def main() -> int:
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    result = validate()
    OUTPUT.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
    return 1 if result["failures"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
