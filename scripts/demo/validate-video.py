#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]

MIN_DURATIONS = {
    "scene-01-stack-config": 14.0,
    "scene-02-local-setup": 12.0,
    "scene-03-example-config": 28.0,
    "scene-04-jupiter-live": 14.0,
    "scene-05-token-live": 18.0,
    "scene-06-observability": 30.0,
    "scene-07-clickhouse-play": 30.0,
}


@dataclass
class FrameStats:
    mean: float
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
    total_luminance = 0.0
    nonblack = 0
    for index in range(0, len(pixels), 3):
        r, g, b = pixels[index], pixels[index + 1], pixels[index + 2]
        luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b
        total_luminance += luminance
        if luminance > 18:
            nonblack += 1
    total = width * height
    return FrameStats(
        mean=total_luminance / total,
        nonblack_ratio=nonblack / total,
    )


def validate_video(path: Path, scene: str) -> dict:
    dur = duration(path)
    minimum = MIN_DURATIONS.get(scene, 5.0)
    stats = frame_at(path, max(0.2, dur * 0.50))

    errors = []
    if dur < minimum:
        errors.append(f"duration {dur:.1f}s below minimum {minimum:.1f}s")
    if stats.nonblack_ratio < 0.05:
        errors.append(f"near-black video; nonblack ratio {stats.nonblack_ratio:.3f}")

    return {
        "scene": scene,
        "path": str(path.relative_to(ROOT)),
        "duration": dur,
        "minimum_duration": minimum,
        "mean_brightness": stats.mean,
        "nonblack_ratio": stats.nonblack_ratio,
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
    parser.add_argument("--report", default="demo-artifacts/review-human/logs/video-validation.json")
    parser.add_argument("--contact-sheet", default="demo-artifacts/review-human/frame-contact-sheet.jpg")
    args = parser.parse_args()

    video_dir = ROOT / args.video_dir
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

    make_contact_sheet(video_dir, contact_sheet)

    result = {
        "videos": videos,
        "contact_sheet": str(contact_sheet.relative_to(ROOT)),
    }
    report_path.write_text(json.dumps(result, indent=2))

    failures = [item for item in videos if item["status"] != "ok"]
    if failures:
        print(json.dumps({"status": "failed", "failures": failures}, indent=2))
        return 1
    print(json.dumps({"status": "ok", "report": str(report_path.relative_to(ROOT)), "contact_sheet": str(contact_sheet.relative_to(ROOT))}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
