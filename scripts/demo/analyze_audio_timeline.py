#!/usr/bin/env python3
from __future__ import annotations

import argparse
import importlib.util
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
REVIEW_DIR = ROOT / "demo-artifacts/review-human"
AUDIO_ANALYSIS_DIR = REVIEW_DIR / "audio-analysis"


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=False)


def ffprobe_duration(audio: Path) -> float:
    proc = run(
        [
            "ffprobe",
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=nokey=1:noprint_wrappers=1",
            str(audio),
        ]
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or f"ffprobe failed for {audio}")
    return float(proc.stdout.strip())


def detected_silences(audio: Path, *, noise_db: str, min_silence: float) -> list[dict[str, float]]:
    proc = run(
        [
            "ffmpeg",
            "-hide_banner",
            "-nostats",
            "-i",
            str(audio),
            "-af",
            f"silencedetect=noise={noise_db}:d={min_silence}",
            "-f",
            "null",
            "-",
        ]
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or f"ffmpeg silencedetect failed for {audio}")

    silences: list[dict[str, float]] = []
    pending_start: float | None = None
    for line in proc.stderr.splitlines():
        start_match = re.search(r"silence_start:\s*([0-9.]+)", line)
        if start_match:
            pending_start = float(start_match.group(1))
            continue
        end_match = re.search(r"silence_end:\s*([0-9.]+)\s*\|\s*silence_duration:\s*([0-9.]+)", line)
        if end_match and pending_start is not None:
            silences.append(
                {
                    "start_seconds": pending_start,
                    "end_seconds": float(end_match.group(1)),
                    "duration_seconds": float(end_match.group(2)),
                }
            )
            pending_start = None
    return silences


def speech_intervals(
    silences: list[dict[str, float]],
    *,
    duration: float,
    min_speech: float,
) -> list[dict[str, float]]:
    intervals: list[dict[str, float]] = []
    cursor = 0.0
    for silence in silences:
        start = float(silence["start_seconds"])
        end = float(silence["end_seconds"])
        if start > cursor and start - cursor >= min_speech:
            intervals.append(
                {
                    "start_seconds": cursor,
                    "end_seconds": start,
                    "duration_seconds": start - cursor,
                }
            )
        cursor = max(cursor, end)
    if duration > cursor and duration - cursor >= min_speech:
        intervals.append(
            {
                "start_seconds": cursor,
                "end_seconds": duration,
                "duration_seconds": duration - cursor,
            }
        )
    return intervals


def group_speech_intervals(
    intervals: list[dict[str, float]],
    *,
    utterance_gap: float,
) -> list[dict[str, Any]]:
    if not intervals:
        return []
    groups: list[list[dict[str, float]]] = [[intervals[0]]]
    for interval in intervals[1:]:
        previous = groups[-1][-1]
        gap = float(interval["start_seconds"]) - float(previous["end_seconds"])
        if gap <= utterance_gap:
            groups[-1].append(interval)
        else:
            groups.append([interval])

    result: list[dict[str, Any]] = []
    for group in groups:
        start = float(group[0]["start_seconds"])
        end = float(group[-1]["end_seconds"])
        result.append(
            {
                "start_seconds": start,
                "end_seconds": end,
                "duration_seconds": end - start,
                "speech_intervals": group,
            }
        )
    return result


def load_voice_module() -> Any:
    path = ROOT / "scripts/demo/voice.py"
    spec = importlib.util.spec_from_file_location("demo_voice", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"Cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.load_env()
    return module


def transcript_entries(scene: str) -> list[dict[str, Any]]:
    voice = load_voice_module()
    transcript = voice.gemini_tutorial_transcript(voice.scene_text(scene))
    entries: list[dict[str, Any]] = []
    prompt_tags: list[str] = []
    for raw in transcript.splitlines():
        line = raw.strip()
        if not line:
            continue
        match = re.match(r"^\[([^\]]+)\]\s*(.*)$", line)
        if match:
            tag = match.group(1).strip()
            text = match.group(2).strip()
        else:
            tag = ""
            text = line
        if tag and tag not in prompt_tags:
            prompt_tags.append(tag)
        entries.append({"prompt_tag": tag, "text": text, "prompt_line": line})
    return entries


def spoken_entries(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [entry for entry in entries if entry.get("text")]


def assign_text(groups: list[dict[str, Any]], entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    utterances: list[dict[str, Any]] = []
    for index, group in enumerate(groups):
        entry = entries[index] if index < len(entries) else {}
        tag = str(entry.get("prompt_tag", ""))
        utterances.append(
            {
                **group,
                "index": index + 1,
                "prompt_tag": tag,
                "prompt_tags": [tag] if tag else [],
                "prompt_line": entry.get("prompt_line", ""),
                "text": entry.get("text", ""),
                "text_source": "scene transcript",
            }
        )
    if len(entries) > len(groups) and utterances:
        tail = entries[len(groups) :]
        remainder = " ".join(str(entry.get("text", "")) for entry in tail if entry.get("text"))
        tail_tags = [str(entry.get("prompt_tag", "")) for entry in tail if entry.get("prompt_tag")]
        prompt_tags = list(dict.fromkeys([*utterances[-1].get("prompt_tags", []), *tail_tags]))
        utterances[-1]["text"] = f'{utterances[-1]["text"]} {remainder}'.strip()
        utterances[-1]["prompt_tags"] = prompt_tags
        utterances[-1]["prompt_tag"] = ", ".join(prompt_tags)
        utterances[-1]["text_source"] = "scene transcript, combined tail"
    return utterances


def analyze(args: argparse.Namespace) -> dict[str, Any]:
    audio = (ROOT / args.audio).resolve() if not Path(args.audio).is_absolute() else Path(args.audio)
    duration = ffprobe_duration(audio)
    silences = detected_silences(audio, noise_db=args.noise_db, min_silence=args.min_silence)
    intervals = speech_intervals(silences, duration=duration, min_speech=args.min_speech)
    groups = group_speech_intervals(intervals, utterance_gap=args.utterance_gap)
    entries = transcript_entries(args.scene)
    utterances = assign_text(groups, spoken_entries(entries))

    for utterance in utterances:
        utterance["timeline_start_seconds"] = float(args.timeline_offset) + float(utterance["start_seconds"])
        utterance["timeline_end_seconds"] = float(args.timeline_offset) + float(utterance["end_seconds"])
    for interval in intervals:
        interval["timeline_start_seconds"] = float(args.timeline_offset) + float(interval["start_seconds"])
        interval["timeline_end_seconds"] = float(args.timeline_offset) + float(interval["end_seconds"])

    return {
        "audio": str(audio.relative_to(ROOT)),
        "scene": args.scene,
        "provider": args.provider,
        "timeline_offset_seconds": float(args.timeline_offset),
        "duration_seconds": duration,
        "method": "ffmpeg silencedetect plus forced alignment to the known generated scene transcript; this is not ASR",
        "prompt_tags": list(dict.fromkeys(str(entry["prompt_tag"]) for entry in entries if entry.get("prompt_tag"))),
        "prompt_transcript": [entry["prompt_line"] for entry in entries],
        "parameters": {
            "noise_db": args.noise_db,
            "min_silence": args.min_silence,
            "min_speech": args.min_speech,
            "utterance_gap": args.utterance_gap,
        },
        "silences": silences,
        "speech_intervals": intervals,
        "utterances": utterances,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--audio", default="demo-artifacts/audio/gemini-smoke-scene-01-readiness-checks.wav")
    parser.add_argument("--scene", default="scene-01-readiness-checks")
    parser.add_argument("--provider", default="gemini")
    parser.add_argument("--timeline-offset", type=float, default=0.0)
    parser.add_argument("--noise-db", default="-35dB")
    parser.add_argument("--min-silence", type=float, default=0.15)
    parser.add_argument("--min-speech", type=float, default=0.12)
    parser.add_argument("--utterance-gap", type=float, default=0.85)
    parser.add_argument("--output", default="")
    args = parser.parse_args()

    output = Path(args.output) if args.output else AUDIO_ANALYSIS_DIR / f"{Path(args.audio).stem}.json"
    if not output.is_absolute():
        output = ROOT / output
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(analyze(args), indent=2) + "\n")
    print(output.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
