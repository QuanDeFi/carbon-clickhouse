#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
import textwrap
from dataclasses import dataclass
from pathlib import Path

import yaml


ROOT = Path(__file__).resolve().parents[2]
REVIEW_DIR = ROOT / "demo-artifacts/review-human"
VIDEOS_DIR = REVIEW_DIR / "videos"
SUBTITLE_DIR = REVIEW_DIR / "subtitles"
FINAL_VIDEO = VIDEOS_DIR / "clickhouse-sink-tutorial-human-no-audio.mp4"
SUBTITLED_VIDEO = VIDEOS_DIR / "clickhouse-sink-tutorial-human-no-audio-subtitled.mp4"


@dataclass
class Cue:
    index: int
    start: float
    end: float
    text: str
    scene: str


def runbook() -> dict:
    return yaml.safe_load((ROOT / "scripts/demo/runbook.yaml").read_text())


def scene_ids() -> list[str]:
    return [scene["id"] for scene in runbook()["scenes"]]


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
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    return float(result.stdout.strip())


def clean_markdown(path: Path) -> str:
    lines: list[str] = []
    in_fence = False
    for raw in path.read_text().splitlines():
        line = raw.strip()
        if line.startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence or line.startswith("#"):
            continue
        if not line:
            lines.append("")
            continue
        line = re.sub(r"`([^`]+)`", r"\1", line)
        line = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", line)
        line = re.sub(r"[*_]{1,2}([^*_]+)[*_]{1,2}", r"\1", line)
        lines.append(line)
    paragraphs = []
    current: list[str] = []
    for line in lines:
        if not line:
            if current:
                paragraphs.append(" ".join(current))
                current = []
            continue
        current.append(line)
    if current:
        paragraphs.append(" ".join(current))
    return "\n\n".join(paragraphs)


def sentence_chunks(text: str, *, max_words: int = 13, max_chars: int = 84) -> list[str]:
    sentences = re.split(r"(?<=[.!?])\s+", text.replace("\n", " ").strip())
    chunks: list[str] = []
    for sentence in sentences:
        words = sentence.split()
        if not words:
            continue
        current: list[str] = []
        for word in words:
            candidate = current + [word]
            if current and (len(candidate) > max_words or len(" ".join(candidate)) > max_chars):
                chunks.append(" ".join(current))
                current = [word]
            else:
                current = candidate
        if current:
            chunks.append(" ".join(current))
    return chunks


def wrap_caption(text: str) -> str:
    return "\n".join(textwrap.wrap(text, width=46, max_lines=2, placeholder="..."))


def make_scene_cues(scene: str, scene_start: float, scene_duration: float, first_index: int) -> list[Cue]:
    script = ROOT / "docs/tutorial-video" / f"{scene}.md"
    chunks = sentence_chunks(clean_markdown(script))
    if not chunks:
        return []
    lead = min(0.6, scene_duration * 0.08)
    tail = min(0.6, scene_duration * 0.08)
    available = max(0.5, scene_duration - lead - tail)
    weights = [max(4, len(chunk.split())) for chunk in chunks]
    total = sum(weights)
    cursor = scene_start + lead
    cues: list[Cue] = []
    for offset, (chunk, weight) in enumerate(zip(chunks, weights), start=0):
        cue_duration = available * weight / total
        start = cursor
        end = scene_start + scene_duration - tail if offset == len(chunks) - 1 else cursor + cue_duration
        if end - start < 1.0:
            end = min(scene_start + scene_duration - tail, start + 1.0)
        cues.append(Cue(first_index + offset, start, end, wrap_caption(chunk), scene))
        cursor = end
    return cues


def srt_time(seconds: float) -> str:
    millis = int(round(seconds * 1000))
    hours, remainder = divmod(millis, 3_600_000)
    minutes, remainder = divmod(remainder, 60_000)
    secs, millis = divmod(remainder, 1000)
    return f"{hours:02}:{minutes:02}:{secs:02},{millis:03}"


def vtt_time(seconds: float) -> str:
    return srt_time(seconds).replace(",", ".")


def write_srt(cues: list[Cue], path: Path) -> None:
    parts = []
    for cue in cues:
        parts.append(f"{cue.index}\n{srt_time(cue.start)} --> {srt_time(cue.end)}\n{cue.text}\n")
    path.write_text("\n".join(parts))


def write_vtt(cues: list[Cue], path: Path) -> None:
    parts = ["WEBVTT\n"]
    for cue in cues:
        parts.append(f"{vtt_time(cue.start)} --> {vtt_time(cue.end)}\n{cue.text}\n")
    path.write_text("\n".join(parts))


def build_cues() -> tuple[list[Cue], dict]:
    cues: list[Cue] = []
    offset = 0.0
    report = {"scenes": []}
    for scene in scene_ids():
        video = VIDEOS_DIR / f"{scene}.mp4"
        if not video.is_file():
            raise FileNotFoundError(video)
        scene_duration = duration(video)
        scene_cues = make_scene_cues(scene, offset, scene_duration, len(cues) + 1)
        cues.extend(scene_cues)
        report["scenes"].append(
            {
                "scene": scene,
                "start_seconds": round(offset, 3),
                "duration_seconds": round(scene_duration, 3),
                "cue_count": len(scene_cues),
            }
        )
        offset += scene_duration
    report["total_cues"] = len(cues)
    report["total_duration_seconds"] = round(offset, 3)
    return cues, report


def ffmpeg_subtitle_path(path: Path) -> str:
    # FFmpeg subtitles filter needs ':' and apostrophes escaped in filenames.
    return str(path.resolve()).replace("\\", "\\\\").replace(":", "\\:").replace("'", "\\'")


def burn_subtitles(srt: Path, output: Path) -> None:
    if not FINAL_VIDEO.is_file():
        raise FileNotFoundError(FINAL_VIDEO)
    style = ",".join(
        [
            "FontName=DejaVu Sans",
            "FontSize=10",
            "PrimaryColour=&H00FFFFFF",
            "OutlineColour=&H00000000",
            "BackColour=&HA8000000",
            "BorderStyle=4",
            "Outline=1",
            "Shadow=0",
            "Alignment=2",
            "MarginV=10",
        ]
    )
    subprocess.run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            str(FINAL_VIDEO),
            "-vf",
            f"subtitles='{ffmpeg_subtitle_path(srt)}':force_style='{style}'",
            "-an",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-pix_fmt",
            "yuv420p",
            str(output),
        ],
        cwd=ROOT,
        check=True,
    )


def generate(*, burn: bool) -> dict:
    SUBTITLE_DIR.mkdir(parents=True, exist_ok=True)
    cues, report = build_cues()
    srt = SUBTITLE_DIR / "clickhouse-sink-tutorial-human-no-audio.srt"
    vtt = SUBTITLE_DIR / "clickhouse-sink-tutorial-human-no-audio.vtt"
    write_srt(cues, srt)
    write_vtt(cues, vtt)
    report.update(
        {
            "srt": str(srt.relative_to(ROOT)),
            "vtt": str(vtt.relative_to(ROOT)),
            "source_video": str(FINAL_VIDEO.relative_to(ROOT)),
        }
    )
    if burn:
        burn_subtitles(srt, SUBTITLED_VIDEO)
        report["subtitled_video"] = str(SUBTITLED_VIDEO.relative_to(ROOT))
        report["subtitled_video_duration_seconds"] = round(duration(SUBTITLED_VIDEO), 3)
    report_path = SUBTITLE_DIR / "subtitle-report.json"
    report_path.write_text(json.dumps(report, indent=2))
    report["report"] = str(report_path.relative_to(ROOT))
    return report


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--no-burn", action="store_true", help="Only write SRT/VTT sidecars.")
    args = parser.parse_args()
    report = generate(burn=not args.no_burn)
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
