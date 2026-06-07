#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[2]


def load_env() -> None:
    for path in (ROOT / ".env.demo.local", ROOT / "scripts/demo/.env.demo.local"):
        if not path.is_file():
            continue
        for raw in path.read_text(errors="ignore").splitlines():
            line = raw.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, value = line.split("=", 1)
            os.environ.setdefault(key.strip(), value.strip().strip('"').strip("'"))
        break


def duration(path: Path) -> float:
    result = subprocess.run(
        [
            "ffprobe",
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=noprint_wrappers=1:nokey=1",
            str(path),
        ],
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(result.stderr)
    return float(result.stdout.strip())


def recording_for(scene: str, recordings: Path) -> Path | None:
    for suffix in (".mp4", ".mkv", ".mov", ".webm"):
        path = recordings / f"{scene}{suffix}"
        if path.is_file():
            return path
    return None


def audio_for(scene: str, audio_dir: Path) -> Path | None:
    for suffix in (".mp3", ".wav"):
        path = audio_dir / f"{scene}{suffix}"
        if path.is_file():
            return path
    return None


def assemble_scene(scene: str) -> int:
    recordings = ROOT / os.environ.get("DEMO_RECORDING_DIR", "demo-artifacts/recordings")
    audio_dir = ROOT / os.environ.get("DEMO_AUDIO_DIR", "demo-artifacts/audio")
    final_dir = ROOT / "demo-artifacts/final-scenes"
    final_dir.mkdir(parents=True, exist_ok=True)
    recording = recording_for(scene, recordings)
    audio = audio_for(scene, audio_dir)
    output = final_dir / f"{scene}.mp4"

    if audio is not None and recording is None:
        cmd = [
            "ffmpeg",
            "-y",
            "-f",
            "lavfi",
            "-i",
            f"color=size={os.environ.get('DEMO_SCREEN_SIZE', '1920x1080')}:rate={os.environ.get('DEMO_FPS', '30')}:color=black",
            "-i",
            str(audio),
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-shortest",
            str(output),
        ]
    elif audio is not None and recording is not None:
        if duration(audio) > duration(recording) + 0.5:
            print(
                f"audio for {scene} is longer than video; refusing to truncate narration",
                file=sys.stderr,
            )
            return 1
        cmd = [
            "ffmpeg",
            "-y",
            "-i",
            str(recording),
            "-i",
            str(audio),
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-shortest",
            str(output),
        ]
    elif recording is not None:
        cmd = ["ffmpeg", "-y", "-i", str(recording), "-c", "copy", str(output)]
    else:
        print(f"missing recording/audio for {scene}", file=sys.stderr)
        return 1

    subprocess.run(cmd, check=True)
    print(f"scene_written: {output.relative_to(ROOT)}")
    return 0


def scene_ids() -> list[str]:
    data = yaml.safe_load((ROOT / "scripts/demo/runbook.yaml").read_text())
    return [scene["id"] for scene in data["scenes"]]


def assemble_all() -> int:
    code = 0
    for scene in scene_ids():
        code = assemble_scene(scene) or code
    if code:
        return code
    final_dir = ROOT / "demo-artifacts/final-scenes"
    concat = ROOT / "demo-artifacts/tmp/final-scenes.txt"
    concat.parent.mkdir(parents=True, exist_ok=True)
    lines = []
    for scene in scene_ids():
        path = final_dir / f"{scene}.mp4"
        if path.is_file():
            lines.append(f"file '{path}'")
    concat.write_text("\n".join(lines) + "\n")
    output = ROOT / os.environ.get("DEMO_FINAL_VIDEO", "demo-artifacts/clickhouse-sink-tutorial.mp4")
    subprocess.run(
        ["ffmpeg", "-y", "-f", "concat", "-safe", "0", "-i", str(concat), "-c", "copy", str(output)],
        check=True,
    )
    print(f"final_video_written: {output.relative_to(ROOT)}")
    return 0


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)
    scene = sub.add_parser("scene")
    scene.add_argument("scene")
    sub.add_parser("all")
    args = parser.parse_args()
    if args.cmd == "scene":
        return assemble_scene(args.scene)
    return assemble_all()


if __name__ == "__main__":
    raise SystemExit(main())
