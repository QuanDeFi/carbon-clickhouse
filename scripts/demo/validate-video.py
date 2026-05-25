#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
import shutil
import statistics
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]

MIN_DURATIONS = {
    "scene-01-intro": 5.0,
    "scene-02-local-setup": 12.0,
    "scene-03-jupiter-ingestion": 16.0,
    "scene-04-clickhouse-validation": 9.0,
    "scene-05-token-program": 18.0,
    "scene-06-async-inserts": 16.0,
    "scene-07-observability": 12.0,
    "scene-08-production-boundaries": 6.0,
}


@dataclass
class FrameStats:
    mean: float
    stdev: float
    nonblack_ratio: float


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True, check=True)


def duration(path: Path) -> float:
    result = run([
        "ffprobe",
        "-v",
        "error",
        "-show_entries",
        "format=duration",
        "-of",
        "default=noprint_wrappers=1:nokey=1",
        str(path),
    ])
    return float(result.stdout.strip())


def read_ppm(data: bytes) -> tuple[int, int, bytes]:
    if not data.startswith(b"P6\n"):
        raise ValueError("not a binary PPM frame")
    parts = data.split(b"\n", 3)
    width, height = [int(part) for part in parts[1].split()]
    max_value = int(parts[2])
    if max_value != 255:
        raise ValueError("unsupported PPM max value")
    return width, height, parts[3]


def frame_at(path: Path, offset: float) -> FrameStats:
    result = subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-ss",
            f"{offset:.3f}",
            "-i",
            str(path),
            "-frames:v",
            "1",
            "-vf",
            "scale=320:-1",
            "-f",
            "image2pipe",
            "-vcodec",
            "ppm",
            "-",
        ],
        cwd=ROOT,
        capture_output=True,
        check=True,
    )
    width, height, pixels = read_ppm(result.stdout)
    values = []
    nonblack = 0
    for index in range(0, len(pixels), 3):
        r, g, b = pixels[index], pixels[index + 1], pixels[index + 2]
        luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b
        values.append(luminance)
        if luminance > 18:
            nonblack += 1
    total = width * height
    return FrameStats(
        mean=statistics.fmean(values),
        stdev=statistics.pstdev(values),
        nonblack_ratio=nonblack / total,
    )


def validate_video(path: Path, scene: str) -> dict:
    dur = duration(path)
    minimum = MIN_DURATIONS.get(scene, 5.0)
    offsets = [max(0.2, dur * 0.10), max(0.2, dur * 0.50), max(0.2, dur * 0.90)]
    stats = [frame_at(path, offset) for offset in offsets]
    max_nonblack = max(stat.nonblack_ratio for stat in stats)
    max_stdev = max(stat.stdev for stat in stats)
    mean_brightness = statistics.fmean(stat.mean for stat in stats)

    errors = []
    if dur < minimum:
        errors.append(f"duration {dur:.1f}s below minimum {minimum:.1f}s")
    if max_nonblack < 0.05:
        errors.append(f"near-black video; max nonblack ratio {max_nonblack:.3f}")
    if max_stdev < 6.0:
        errors.append(f"low visual variation; max stdev {max_stdev:.2f}")

    return {
        "scene": scene,
        "path": str(path.relative_to(ROOT)),
        "duration": dur,
        "minimum_duration": minimum,
        "mean_brightness": mean_brightness,
        "max_nonblack_ratio": max_nonblack,
        "max_stdev": max_stdev,
        "status": "ok" if not errors else "failed",
        "errors": errors,
    }


def screenshot_stats(path: Path) -> dict:
    stats = frame_at(path, 0)
    errors = []
    if stats.nonblack_ratio < 0.05:
        errors.append("near-black screenshot")
    if stats.stdev < 6.0:
        errors.append("low visual variation")
    return {
        "path": str(path.relative_to(ROOT)),
        "mean_brightness": stats.mean,
        "nonblack_ratio": stats.nonblack_ratio,
        "stdev": stats.stdev,
        "status": "ok" if not errors else "failed",
        "errors": errors,
    }


def make_contact_sheet(video_dir: Path, output: Path) -> None:
    scenes = sorted(path for path in video_dir.glob("scene-*.mp4"))
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        for index, path in enumerate(scenes, start=1):
            dur = duration(path)
            frame = tmp_path / f"{index:02d}-{path.stem}.jpg"
            subprocess.run(
                [
                    "ffmpeg",
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-y",
                    "-ss",
                    f"{max(0.2, dur * 0.5):.3f}",
                    "-i",
                    str(path),
                    "-frames:v",
                    "1",
                    "-vf",
                    "scale=460:-1",
                    str(frame),
                ],
                cwd=ROOT,
                check=True,
            )
        subprocess.run(
            [
                "ffmpeg",
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-pattern_type",
                "glob",
                "-i",
                str(tmp_path / "*.jpg"),
                "-vf",
                "tile=4x2",
                str(output),
            ],
            cwd=ROOT,
            check=True,
        )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--video-dir", default="demo-artifacts/review-human/videos")
    parser.add_argument("--screenshots", default="demo-artifacts/screenshots")
    parser.add_argument("--report", default="demo-artifacts/review-human/logs/video-validation.json")
    parser.add_argument("--contact-sheet", default="demo-artifacts/review-human/frame-contact-sheet.jpg")
    args = parser.parse_args()

    video_dir = ROOT / args.video_dir
    screenshot_dir = ROOT / args.screenshots
    report_path = ROOT / args.report
    contact_sheet = ROOT / args.contact_sheet
    report_path.parent.mkdir(parents=True, exist_ok=True)

    videos = []
    for scene, minimum in MIN_DURATIONS.items():
        path = video_dir / f"{scene}.mp4"
        if not path.is_file():
            videos.append({"scene": scene, "path": str(path.relative_to(ROOT)), "status": "failed", "errors": ["missing video"]})
            continue
        videos.append(validate_video(path, scene))

    screenshots = []
    if screenshot_dir.is_dir():
        for path in sorted(screenshot_dir.glob("*.png")):
            screenshots.append(screenshot_stats(path))

    make_contact_sheet(video_dir, contact_sheet)

    result = {
        "videos": videos,
        "screenshots": screenshots,
        "contact_sheet": str(contact_sheet.relative_to(ROOT)),
    }
    report_path.write_text(json.dumps(result, indent=2))

    failures = [item for item in videos + screenshots if item["status"] != "ok"]
    if failures:
        print(json.dumps({"status": "failed", "failures": failures}, indent=2))
        return 1
    print(json.dumps({"status": "ok", "report": str(report_path.relative_to(ROOT)), "contact_sheet": str(contact_sheet.relative_to(ROOT))}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
