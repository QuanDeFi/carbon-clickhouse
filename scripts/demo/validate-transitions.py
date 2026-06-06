#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
REVIEW_DIR = ROOT / "demo-artifacts/review-human"
LOG_DIR = REVIEW_DIR / "logs"
SUBTITLED_VIDEO = REVIEW_DIR / "videos/clickhouse-sink-tutorial-human-no-audio-subtitled.mp4"
SOURCE_VIDEO = REVIEW_DIR / "videos/clickhouse-sink-tutorial-human-no-audio.mp4"
VIDEO = SUBTITLED_VIDEO if SUBTITLED_VIDEO.is_file() else SOURCE_VIDEO
OUTPUT = LOG_DIR / "transition-validation.json"

SAMPLE_FPS = 30
SAMPLE_WIDTH = 480
SAMPLE_HEIGHT = 270
PRE_SECONDS = 0.4
POST_SECONDS = 0.55
BRIGHT_MEAN_THRESHOLD = 70.0
WHITE_PIXEL_RATIO_THRESHOLD = 0.20
MID_PIXEL_RATIO_THRESHOLD = 0.35
MAX_BLACK_INTERVAL_SECONDS = 1.25


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


def frame_metrics(data: bytes) -> dict[str, float]:
    total = len(data) // 3
    luminance = [
        0.2126 * data[index] + 0.7152 * data[index + 1] + 0.0722 * data[index + 2]
        for index in range(0, len(data), 3)
    ]
    return {
        "mean_luminance": sum(luminance) / total,
        "white_pixel_ratio": sum(1 for value in luminance if value > 210) / total,
        "mid_pixel_ratio": sum(1 for value in luminance if value > 120) / total,
    }


def sample_transition(video: Path, interval: dict[str, float]) -> list[dict[str, float]]:
    start = max(0.0, interval["start"] - PRE_SECONDS)
    duration = (interval["end"] - start) + POST_SECONDS
    proc = subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            f"{start:.3f}",
            "-t",
            f"{duration:.3f}",
            "-i",
            str(video),
            "-vf",
            f"fps={SAMPLE_FPS},scale={SAMPLE_WIDTH}:{SAMPLE_HEIGHT},format=rgb24",
            "-f",
            "rawvideo",
            "-",
        ],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        check=True,
    )
    frame_size = SAMPLE_WIDTH * SAMPLE_HEIGHT * 3
    frames = []
    for index in range(0, len(proc.stdout) // frame_size):
        chunk = proc.stdout[index * frame_size : (index + 1) * frame_size]
        metrics = frame_metrics(chunk)
        frames.append({"seconds": start + index / SAMPLE_FPS, **metrics})
    return frames


def bright_outliers(frames: list[dict[str, float]]) -> list[dict[str, float]]:
    outliers = []
    for frame in frames:
        if (
            frame["mean_luminance"] >= BRIGHT_MEAN_THRESHOLD
            or frame["white_pixel_ratio"] >= WHITE_PIXEL_RATIO_THRESHOLD
            or frame["mid_pixel_ratio"] >= MID_PIXEL_RATIO_THRESHOLD
        ):
            outliers.append(
                {
                    "seconds": round(frame["seconds"], 3),
                    "mean_luminance": round(frame["mean_luminance"], 2),
                    "white_pixel_ratio": round(frame["white_pixel_ratio"], 4),
                    "mid_pixel_ratio": round(frame["mid_pixel_ratio"], 4),
                }
            )
    return outliers


def validate() -> dict[str, object]:
    if not VIDEO.is_file():
        raise FileNotFoundError(VIDEO)
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    intervals = [
        interval
        for interval in run_blackdetect(VIDEO)
        if interval["duration"] <= MAX_BLACK_INTERVAL_SECONDS
    ]
    checks = []
    failures = []
    for index, interval in enumerate(intervals, start=1):
        frames = sample_transition(VIDEO, interval)
        outliers = bright_outliers(frames)
        check = {
            "name": f"transition-{index}:bright-flicker",
            "black_interval": {
                "start_seconds": round(interval["start"], 3),
                "end_seconds": round(interval["end"], 3),
                "duration_seconds": round(interval["duration"], 3),
            },
            "sample_window_seconds": [
                round(max(0.0, interval["start"] - PRE_SECONDS), 3),
                round(interval["end"] + POST_SECONDS, 3),
            ],
            "bright_outliers": outliers,
        }
        checks.append(check)
        if outliers:
            failures.append({**check, "reason": "bright frame detected near transition"})
    return {
        "status": "ok" if not failures else "failed",
        "video": str(VIDEO.relative_to(ROOT)),
        "thresholds": {
            "sample_fps": SAMPLE_FPS,
            "pre_seconds": PRE_SECONDS,
            "post_seconds": POST_SECONDS,
            "bright_mean_threshold": BRIGHT_MEAN_THRESHOLD,
            "white_pixel_ratio_threshold": WHITE_PIXEL_RATIO_THRESHOLD,
            "mid_pixel_ratio_threshold": MID_PIXEL_RATIO_THRESHOLD,
        },
        "checks": checks,
        "failures": failures,
        "report": str(OUTPUT.relative_to(ROOT)),
    }


def main() -> int:
    result = validate()
    OUTPUT.write_text(json.dumps(result, indent=2))
    print(json.dumps(result, indent=2))
    return 0 if result["status"] == "ok" else 1


if __name__ == "__main__":
    sys.exit(main())
