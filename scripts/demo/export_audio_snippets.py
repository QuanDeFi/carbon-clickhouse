#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]
REVIEW_DIR = ROOT / "demo-artifacts/review-human"
AUDIO_ANALYSIS_DIR = REVIEW_DIR / "audio-analysis"
DEFAULT_OUTPUT_DIR = REVIEW_DIR / "audio-snippets"


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, check=False)


def require_ok(proc: subprocess.CompletedProcess[str], label: str) -> None:
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip()
        raise RuntimeError(f"{label} failed: {detail}")


def load_json(path: Path) -> Any:
    return json.loads(path.read_text())


def write_json(path: Path, data: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2) + "\n")


def rel(path: Path) -> str:
    path = path.resolve()
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def root_path(value: str | Path) -> Path:
    path = Path(value)
    return path if path.is_absolute() else ROOT / path


def selected_analysis_files(scenes: str) -> list[Path]:
    paths = sorted(AUDIO_ANALYSIS_DIR.glob("scene-*.json"))
    if not scenes:
        return paths
    wanted = {scene.strip() for scene in scenes.split(",") if scene.strip()}
    selected = [path for path in paths if path.stem in wanted]
    missing = sorted(wanted - {path.stem for path in selected})
    if missing:
        raise RuntimeError("Missing audio analysis for scene ids: " + ", ".join(missing))
    return selected


def snippet_id(scene: str, index: int) -> str:
    return f"{scene}--u{index:02d}"


def cut_clip(
    source: Path,
    output: Path,
    *,
    start: float,
    end: float,
    sample_rate: int,
    channels: int,
) -> None:
    duration = max(0.001, end - start)
    output.parent.mkdir(parents=True, exist_ok=True)
    proc = run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-ss",
            f"{start:.6f}",
            "-i",
            str(source),
            "-t",
            f"{duration:.6f}",
            "-vn",
            "-ac",
            str(channels),
            "-ar",
            str(sample_rate),
            "-c:a",
            "pcm_s16le",
            str(output),
        ]
    )
    require_ok(proc, f"cut {output}")


def export_snippets(args: argparse.Namespace) -> dict[str, Any]:
    output_dir = root_path(args.output_dir)
    snippets: list[dict[str, Any]] = []
    for analysis_path in selected_analysis_files(args.scenes):
        analysis = load_json(analysis_path)
        scene = str(analysis.get("scene") or analysis_path.stem)
        source_audio = root_path(str(analysis.get("audio", "")))
        if not source_audio.is_file():
            raise RuntimeError(f"Source audio missing for {scene}: {source_audio}")
        scene_dir = output_dir / scene
        for utterance in analysis.get("utterances", []):
            if not str(utterance.get("text", "")).strip():
                continue
            index = int(utterance.get("index", len(snippets) + 1))
            start = max(0.0, float(utterance.get("start_seconds", 0.0)) - args.padding_before)
            end = min(
                float(analysis.get("duration_seconds", 0.0) or 0.0),
                float(utterance.get("end_seconds", start)) + args.padding_after,
            )
            sid = snippet_id(scene, index)
            clip = scene_dir / f"{sid}.wav"
            cut_clip(
                source_audio,
                clip,
                start=start,
                end=end,
                sample_rate=args.sample_rate,
                channels=args.channels,
            )
            snippets.append(
                {
                    "snippet_id": sid,
                    "scene": scene,
                    "utterance_index": index,
                    "clip": rel(clip),
                    "source_audio": rel(source_audio),
                    "analysis": rel(analysis_path),
                    "cut_start_seconds": start,
                    "cut_end_seconds": end,
                    "cut_duration_seconds": end - start,
                    "utterance_start_seconds": float(utterance.get("start_seconds", 0.0)),
                    "utterance_end_seconds": float(utterance.get("end_seconds", 0.0)),
                    "prompt_tags": utterance.get("prompt_tags", []),
                    "aligned_text": utterance.get("text", ""),
                    "text_source": utterance.get("text_source", ""),
                    "speech_intervals": utterance.get("speech_intervals", []),
                }
            )
    manifest = {
        "schema": "carbon-clickhouse-demo-snippet-manifest/v1",
        "purpose": "standalone audio clips for external transcription validation of detected scene voiceover snippets",
        "audio_format": {
            "container": "wav",
            "codec": "pcm_s16le",
            "sample_rate_hz": args.sample_rate,
            "channels": args.channels,
        },
        "padding_seconds": {
            "before": args.padding_before,
            "after": args.padding_after,
        },
        "transcription_results_path": rel(output_dir / "snippet-transcriptions.json"),
        "snippets": snippets,
    }
    write_json(output_dir / "snippet-manifest.json", manifest)
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser(description="Export detected scene-audio snippets as standalone WAV clips.")
    parser.add_argument("--scenes", default="", help="Comma-separated scene ids. Defaults to all scene-*.json analyses.")
    parser.add_argument("--output-dir", default=str(DEFAULT_OUTPUT_DIR))
    parser.add_argument("--sample-rate", type=int, default=16000)
    parser.add_argument("--channels", type=int, default=1)
    parser.add_argument("--padding-before", type=float, default=0.08)
    parser.add_argument("--padding-after", type=float, default=0.12)
    args = parser.parse_args()

    manifest = export_snippets(args)
    print(f"exported {len(manifest['snippets'])} snippets")
    print(rel(root_path(args.output_dir) / "snippet-manifest.json"))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
