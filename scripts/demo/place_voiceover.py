#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import analyze_audio_timeline
import build_timeline_review


ROOT = Path(__file__).resolve().parents[2]
REVIEW_DIR = ROOT / "demo-artifacts/review-human"
SUBTITLE_REPORT = REVIEW_DIR / "subtitles/subtitle-report.json"
DEFAULT_VIDEO = REVIEW_DIR / "videos/clickhouse-sink-tutorial-human-no-audio.mp4"
DEFAULT_OUTPUT_WAV = REVIEW_DIR / "audio/voiceover-timeline.wav"
DEFAULT_OUTPUT_VIDEO = REVIEW_DIR / "videos/clickhouse-sink-tutorial-human-voiceover.mp4"
DEFAULT_REPORT = REVIEW_DIR / "audio/voiceover-placement-report.json"
DEFAULT_OVERRIDES = ROOT / "scripts/demo/voiceover-placement-overrides.json"
SUPPORTED_AUDIO_SUFFIXES = (".wav", ".mp3")


@dataclass
class PlacementContext:
    scenes: list[dict[str, Any]]
    cues: list[build_timeline_review.Cue]
    events: list[dict[str, Any]]
    actions: list[dict[str, Any]]
    overrides: dict[str, list[dict[str, Any]]]
    video_duration: float


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(cmd, cwd=ROOT, text=True, capture_output=True, check=False)


def require_ok(proc: subprocess.CompletedProcess[str], label: str) -> None:
    if proc.returncode != 0:
        detail = proc.stderr.strip() or proc.stdout.strip()
        raise RuntimeError(f"{label} failed: {detail}")


def ffprobe_duration(path: Path) -> float:
    proc = run(
        [
            "ffprobe",
            "-v",
            "error",
            "-show_entries",
            "format=duration",
            "-of",
            "default=nokey=1:noprint_wrappers=1",
            str(path),
        ]
    )
    require_ok(proc, f"ffprobe {path}")
    return float(proc.stdout.strip())


def load_json(path: Path, default: Any) -> Any:
    if not path.is_file():
        return default
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


def scene_end(scene: dict[str, Any]) -> float:
    return float(scene["start_seconds"]) + float(scene["duration_seconds"])


def selected_scenes(all_scenes: list[dict[str, Any]], names: str) -> list[dict[str, Any]]:
    if not names:
        return all_scenes
    wanted = {name.strip() for name in names.split(",") if name.strip()}
    selected = [scene for scene in all_scenes if scene["scene"] in wanted]
    missing = sorted(wanted - {scene["scene"] for scene in selected})
    if missing:
        raise RuntimeError("Unknown scene ids: " + ", ".join(missing))
    return selected


def audio_dir() -> Path:
    return root_path(os.environ.get("DEMO_AUDIO_DIR", "demo-artifacts/audio"))


def audio_for_scene(scene_id: str, directory: Path, stem_template: str) -> Path | None:
    rendered = stem_template.format(scene=scene_id)
    candidate = directory / rendered
    if candidate.suffix:
        return candidate if candidate.is_file() else None
    for suffix in SUPPORTED_AUDIO_SUFFIXES:
        path = directory / f"{rendered}{suffix}"
        if path.is_file():
            return path
    return None


def analysis_path_for(audio: Path) -> Path:
    return REVIEW_DIR / "audio-analysis" / f"{audio.stem}.json"


def analysis_needs_refresh(path: Path, audio: Path, scene_start: float, force: bool) -> bool:
    if force or not path.is_file():
        return True
    if path.stat().st_mtime < audio.stat().st_mtime:
        return True
    try:
        data = json.loads(path.read_text())
    except Exception:
        return True
    if not data.get("utterances"):
        return True
    if "prompt_tags" not in data:
        return True
    return abs(float(data.get("timeline_offset_seconds", -9999)) - scene_start) > 0.01


def analyze_audio(audio: Path, scene_id: str, scene_start: float, provider: str, args: argparse.Namespace) -> tuple[dict[str, Any], Path]:
    output = analysis_path_for(audio)
    if analysis_needs_refresh(output, audio, scene_start, args.force_analysis):
        ns = SimpleNamespace(
            audio=rel(audio),
            scene=scene_id,
            provider=provider,
            timeline_offset=scene_start,
            noise_db=args.noise_db,
            min_silence=args.min_silence,
            min_speech=args.min_speech,
            utterance_gap=args.utterance_gap,
        )
        data = analyze_audio_timeline.analyze(ns)
        write_json(output, data)
    else:
        data = json.loads(output.read_text())
    return data, output


def normalize_text(value: str) -> str:
    return " ".join(tokenize(value))


def tokenize(value: str) -> list[str]:
    import re

    return re.findall(r"[a-z0-9]+", value.lower())


def cue_score(text: str, cue: build_timeline_review.Cue) -> float:
    query = tokenize(text)
    candidate = tokenize(cue.text)
    if not query or not candidate:
        return 0.0
    q = " ".join(query)
    c = " ".join(candidate)
    if c and c in q:
        return 1.0
    if q and q in c:
        return 0.98
    overlap = len(set(query) & set(candidate))
    return overlap / max(1, min(len(set(query)), len(set(candidate))))


def best_cue(text: str, scene_id: str, cues: list[build_timeline_review.Cue], fallback_index: int) -> build_timeline_review.Cue | None:
    scene_cues = [cue for cue in cues if cue.scene == scene_id]
    if not scene_cues:
        return None
    scored = [(cue_score(text, cue), cue) for cue in scene_cues]
    scored.sort(key=lambda item: item[0], reverse=True)
    if scored and scored[0][0] >= 0.34:
        return scored[0][1]
    if 0 <= fallback_index < len(scene_cues):
        return scene_cues[fallback_index]
    return scene_cues[-1]


def actions_for_scene(actions: list[dict[str, Any]], scene_id: str) -> list[dict[str, Any]]:
    return [action for action in actions if action["scene"] == scene_id]


def action_at(actions: list[dict[str, Any]], seconds: float) -> dict[str, Any] | None:
    for action in actions:
        if float(action["time"]) <= seconds <= float(action["end"]):
            return action
    if not actions:
        return None
    return min(actions, key=lambda action: abs(float(action["time"]) - seconds))


def action_score(text: str, action: dict[str, Any]) -> float:
    query = tokenize(text)
    candidate = tokenize(f"{action.get('action', '')} {action.get('detail', '')} {action.get('kind', '')}")
    if not query or not candidate:
        return 0.0
    overlap = len(set(query) & set(candidate))
    return overlap / max(1, min(len(set(query)), len(set(candidate))))


def best_action(text: str, actions: list[dict[str, Any]], fallback_index: int) -> dict[str, Any] | None:
    if not actions:
        return None
    scored = [(action_score(text, action), action) for action in actions]
    scored.sort(key=lambda item: item[0], reverse=True)
    if scored and scored[0][0] >= 0.22:
        return scored[0][1]
    return actions[min(fallback_index, len(actions) - 1)]


def action_target(action: dict[str, Any]) -> dict[str, Any]:
    kind = str(action.get("kind", ""))
    return {
        "target_source": "browser-action" if kind == "Browser" else "runbook-action",
        "target_action": action.get("action"),
        "target_action_start_seconds": float(action["time"]),
        "target_action_end_seconds": float(action["end"]),
        "target_action_source": action.get("source"),
        "target_timeline_start_seconds": float(action["time"]),
    }


def events_for_scene(events: list[dict[str, Any]], scene_id: str) -> dict[str, dict[str, Any]]:
    return {str(event["label"]): event for event in events if event["scene"] == scene_id}


def matching_override(overrides: dict[str, list[dict[str, Any]]], scene_id: str, text: str) -> dict[str, Any] | None:
    lowered = text.lower()
    for override in overrides.get(scene_id, []):
        needle = str(override.get("text_contains", "")).lower()
        if needle and needle in lowered:
            return override
    return None


def event_target(
    override: dict[str, Any],
    events_by_label: dict[str, dict[str, Any]],
    scene: dict[str, Any],
) -> tuple[float | None, dict[str, Any]]:
    lead = float(override.get("lead_seconds", 0.0))
    details: dict[str, Any] = {"lead_seconds": lead}
    if "target_event" in override:
        label = str(override["target_event"])
        event = events_by_label.get(label)
        details["target_event"] = label
        details["allow_fallback"] = bool(override.get("allow_fallback", False))
        if not event:
            details["missing_target_event"] = label
            return None, details
        target = float(event["time"]) + lead
        details["target_event_time_seconds"] = float(event["time"])
        details["target_source"] = "browser-event"
        return target, details
    if "target_timeline_start_seconds" in override:
        details["target_source"] = "manual-timeline"
        return float(override["target_timeline_start_seconds"]) + lead, details
    if "target_scene_start_seconds" in override:
        details["target_source"] = "manual-scene-local"
        return float(scene["start_seconds"]) + float(override["target_scene_start_seconds"]) + lead, details
    return None, details


def choose_target(
    *,
    utterance: dict[str, Any],
    utterance_index: int,
    scene: dict[str, Any],
    context: PlacementContext,
) -> dict[str, Any]:
    scene_id = str(scene["scene"])
    text = str(utterance.get("text", ""))
    scene_actions = actions_for_scene(context.actions, scene_id)
    scene_events = events_for_scene(context.events, scene_id)
    override = matching_override(context.overrides, scene_id, text)
    target: float | None = None
    details: dict[str, Any] = {}

    if override:
        target, details = event_target(override, scene_events, scene)
        details["override"] = override
        if target is not None:
            details["target_source"] = details.get("target_source", "manual-override")
            details["locked_to_event"] = bool(details.get("target_event"))

    cue = best_cue(text, scene_id, context.cues, utterance_index - 1)
    fallback_allowed = not (details.get("missing_target_event") and not details.get("allow_fallback"))
    if target is None and fallback_allowed:
        action = best_action(text, scene_actions, utterance_index - 1)
        if action:
            details = {**details, **action_target(action)}
            target = float(details["target_timeline_start_seconds"])

    if cue:
        details.update(
            {
                "subtitle_cue_index": cue.index,
                "subtitle_cue_start_seconds": cue.start,
                "subtitle_cue_text": cue.text,
                "subtitle_delta_seconds": float(details.get("target_timeline_start_seconds", target or cue.start)) - cue.start,
            }
        )

    if target is None and cue and fallback_allowed:
        target = cue.start
        details = {
            **details,
            "target_source": "subtitle-cue",
            "target_timeline_start_seconds": cue.start,
        }

    if target is None:
        target = float(scene["start_seconds"])
        details = {
            **details,
            "target_source": "missing-target-event" if details.get("missing_target_event") else "scene-start",
        }

    unclamped_target = target
    target = max(float(scene["start_seconds"]), target)
    target = max(0.0, target)
    if abs(target - unclamped_target) > 0.001:
        details["clamped_from_seconds"] = unclamped_target

    details["target_timeline_start_seconds"] = target
    return details


def make_placement(
    scene: dict[str, Any],
    audio: Path,
    audio_duration: float,
    analysis: dict[str, Any],
    utterance: dict[str, Any],
    utterance_index: int,
    context: PlacementContext,
    args: argparse.Namespace,
) -> dict[str, Any]:
    target = choose_target(utterance=utterance, utterance_index=utterance_index, scene=scene, context=context)
    speech_start = float(utterance["start_seconds"])
    speech_end = float(utterance["end_seconds"])
    cut_start = max(0.0, speech_start - args.cut_lead)
    cut_end = min(audio_duration, speech_end + args.cut_tail)
    cut_duration = max(0.0, cut_end - cut_start)
    placement = {
        "scene": scene["scene"],
        "utterance_index": utterance_index,
        "prompt_tag": utterance.get("prompt_tag", ""),
        "prompt_tags": utterance.get("prompt_tags", []),
        "text": utterance.get("text", ""),
        "audio": rel(audio),
        "analysis": rel(analysis_path_for(audio)),
        "source_audio_start_seconds": speech_start,
        "source_audio_end_seconds": speech_end,
        "source_cut_start_seconds": cut_start,
        "source_cut_end_seconds": cut_end,
        "clip_duration_seconds": cut_duration,
        "scene_start_seconds": float(scene["start_seconds"]),
        "scene_end_seconds": scene_end(scene),
        **target,
    }
    placement["target_timeline_end_seconds"] = float(placement["target_timeline_start_seconds"]) + cut_duration
    if "target_event_time_seconds" in placement:
        lead = float(placement.get("lead_seconds", 0.0))
        expected = float(placement["target_event_time_seconds"]) + lead
        placement["delta_to_expected_event_lead_seconds"] = float(placement["target_timeline_start_seconds"]) - expected
    return placement


def shift_or_report_overlaps(placements: list[dict[str, Any]], min_gap: float, shift_unlocked: bool) -> list[dict[str, Any]]:
    errors: list[dict[str, Any]] = []
    ordered = sorted(placements, key=lambda item: (float(item["target_timeline_start_seconds"]), item["scene"], item["utterance_index"]))
    previous: dict[str, Any] | None = None
    for placement in ordered:
        if previous is None:
            previous = placement
            continue
        required = float(previous["target_timeline_end_seconds"]) + min_gap
        current = float(placement["target_timeline_start_seconds"])
        if current < required:
            if shift_unlocked and not placement.get("locked_to_event"):
                placement["overlap_shifted_from_seconds"] = current
                placement["target_timeline_start_seconds"] = required
                placement["target_timeline_end_seconds"] = required + float(placement["clip_duration_seconds"])
            else:
                errors.append(
                    {
                        "type": "overlap",
                        "previous_scene": previous["scene"],
                        "previous_utterance_index": previous["utterance_index"],
                        "scene": placement["scene"],
                        "utterance_index": placement["utterance_index"],
                        "overlap_seconds": required - current,
                    }
                )
        previous = placement
    return errors


def validate_placements(placements: list[dict[str, Any]], context: PlacementContext, args: argparse.Namespace) -> list[dict[str, Any]]:
    errors: list[dict[str, Any]] = []
    errors.extend(shift_or_report_overlaps(placements, args.min_gap, not args.no_shift_overlaps))
    for placement in placements:
        if float(placement["clip_duration_seconds"]) <= 0:
            errors.append({"type": "empty-clip", "scene": placement["scene"], "utterance_index": placement["utterance_index"]})
        if placement.get("missing_target_event") and not placement.get("allow_fallback"):
            errors.append(
                {
                    "type": "missing-target-event",
                    "scene": placement["scene"],
                    "utterance_index": placement["utterance_index"],
                    "target_event": placement.get("missing_target_event"),
                }
            )
        if float(placement["target_timeline_end_seconds"]) > context.video_duration + args.duration_tolerance:
            errors.append(
                {
                    "type": "beyond-video",
                    "scene": placement["scene"],
                    "utterance_index": placement["utterance_index"],
                    "target_end_seconds": placement["target_timeline_end_seconds"],
                    "video_duration_seconds": context.video_duration,
                }
            )
        if float(placement["target_timeline_end_seconds"]) > float(placement["scene_end_seconds"]) + args.scene_overrun_tolerance:
            errors.append(
                {
                    "type": "beyond-scene",
                    "scene": placement["scene"],
                    "utterance_index": placement["utterance_index"],
                    "target_end_seconds": placement["target_timeline_end_seconds"],
                    "scene_end_seconds": placement["scene_end_seconds"],
                }
            )
        if "delta_to_expected_event_lead_seconds" in placement:
            delta = abs(float(placement["delta_to_expected_event_lead_seconds"]))
            if delta > args.event_delta:
                errors.append(
                    {
                        "type": "event-delta",
                        "scene": placement["scene"],
                        "utterance_index": placement["utterance_index"],
                        "target_event": placement.get("target_event"),
                        "delta_seconds": delta,
                    }
                )
    return errors


def extract_clip(source: Path, placement: dict[str, Any], output: Path, args: argparse.Namespace) -> None:
    duration = float(placement["clip_duration_seconds"])
    fade_in = min(args.clip_fade_in, max(0.0, duration / 3.0))
    fade_out = min(args.clip_fade_out, max(0.0, duration / 3.0))
    fade_out_start = max(0.0, duration - fade_out)
    filters = [
        f"atrim=start={float(placement['source_cut_start_seconds']):.6f}:end={float(placement['source_cut_end_seconds']):.6f}",
        "asetpts=PTS-STARTPTS",
    ]
    if fade_in > 0:
        filters.append(f"afade=t=in:st=0:d={fade_in:.6f}")
    if fade_out > 0:
        filters.append(f"afade=t=out:st={fade_out_start:.6f}:d={fade_out:.6f}")
    filters.append("aformat=sample_rates=48000:channel_layouts=mono")
    proc = run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            str(source),
            "-vn",
            "-af",
            ",".join(filters),
            "-c:a",
            "pcm_s16le",
            str(output),
        ]
    )
    require_ok(proc, f"extract clip {placement['scene']} #{placement['utterance_index']}")


def render_voiceover(placements: list[dict[str, Any]], output: Path, total_duration: float, args: argparse.Namespace) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    if not placements:
        proc = run(
            [
                "ffmpeg",
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-t",
                f"{total_duration:.6f}",
                "-i",
                "anullsrc=r=48000:cl=mono",
                "-c:a",
                "pcm_s16le",
                str(output),
            ]
        )
        require_ok(proc, "render silent voiceover")
        return

    with tempfile.TemporaryDirectory(prefix="voiceover-clips-", dir=REVIEW_DIR / "audio") as tmp:
        tmp_dir = Path(tmp)
        clip_paths: list[Path] = []
        for index, placement in enumerate(placements, start=1):
            clip = tmp_dir / f"clip-{index:03d}.wav"
            extract_clip(ROOT / placement["audio"], placement, clip, args)
            clip_paths.append(clip)

        cmd = [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-t",
            f"{total_duration:.6f}",
            "-i",
            "anullsrc=r=48000:cl=mono",
        ]
        for clip in clip_paths:
            cmd.extend(["-i", str(clip)])

        filters = [f"[0:a]atrim=0:{total_duration:.6f},asetpts=PTS-STARTPTS[base]"]
        mix_inputs = ["[base]"]
        for index, placement in enumerate(placements, start=1):
            delay_ms = max(0, int(round(float(placement["target_timeline_start_seconds"]) * 1000)))
            filters.append(f"[{index}:a]adelay={delay_ms}:all=1[a{index}]")
            mix_inputs.append(f"[a{index}]")
        filters.append(
            "".join(mix_inputs)
            + f"amix=inputs={len(mix_inputs)}:duration=first:dropout_transition=0:normalize=0,"
            + f"atrim=0:{total_duration:.6f},asetpts=PTS-STARTPTS[aout]"
        )
        cmd.extend(["-filter_complex", ";".join(filters), "-map", "[aout]", "-c:a", "pcm_s16le", str(output)])
        proc = run(cmd)
        require_ok(proc, "render voiceover timeline")


def mux_video(video: Path, voiceover: Path, output: Path) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    proc = run(
        [
            "ffmpeg",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-i",
            str(video),
            "-i",
            str(voiceover),
            "-map",
            "0:v:0",
            "-map",
            "1:a:0",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-movflags",
            "+faststart",
            str(output),
        ]
    )
    require_ok(proc, "mux voiceover video")


def load_context(args: argparse.Namespace) -> PlacementContext:
    subtitle_report = load_json(root_path(args.subtitle_report), {})
    if not subtitle_report.get("scenes"):
        raise RuntimeError(f"Subtitle report missing scenes: {args.subtitle_report}")
    scenes = subtitle_report["scenes"]
    video_duration = ffprobe_duration(root_path(args.video))
    events = build_timeline_review.browser_events(scenes)
    actions = build_timeline_review.runbook_actions(scenes, events)
    cues = build_timeline_review.parse_srt(build_timeline_review.SRT, scenes) if build_timeline_review.SRT.is_file() else []
    overrides = load_json(root_path(args.overrides), {})
    return PlacementContext(
        scenes=scenes,
        cues=cues,
        events=events,
        actions=actions,
        overrides=overrides,
        video_duration=video_duration,
    )


def build_placements(args: argparse.Namespace, context: PlacementContext) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    scenes = selected_scenes(context.scenes, args.scenes)
    directory = root_path(args.audio_dir)
    errors: list[dict[str, Any]] = []
    placements: list[dict[str, Any]] = []
    for scene in scenes:
        scene_id = str(scene["scene"])
        audio = audio_for_scene(scene_id, directory, args.audio_stem_template)
        if audio is None:
            error = {"type": "missing-audio", "scene": scene_id, "audio_stem_template": args.audio_stem_template}
            if args.allow_missing_audio:
                continue
            errors.append(error)
            continue
        audio_duration = ffprobe_duration(audio)
        analysis, _ = analyze_audio(audio, scene_id, float(scene["start_seconds"]), args.provider, args)
        utterances = analysis.get("utterances", [])
        if not utterances:
            errors.append({"type": "no-utterances", "scene": scene_id, "audio": rel(audio)})
            continue
        for index, utterance in enumerate(utterances, start=1):
            placements.append(make_placement(scene, audio, audio_duration, analysis, utterance, index, context, args))
    if any(error["type"] == "missing-audio" for error in errors) and not args.allow_missing_audio:
        return placements, errors
    errors.extend(validate_placements(placements, context, args))
    return sorted(placements, key=lambda item: (float(item["target_timeline_start_seconds"]), item["scene"], item["utterance_index"])), errors


def report_data(args: argparse.Namespace, context: PlacementContext, placements: list[dict[str, Any]], errors: list[dict[str, Any]]) -> dict[str, Any]:
    return {
        "created_at": time.time(),
        "status": "ok" if not errors else "failed",
        "video": rel(root_path(args.video)),
        "video_duration_seconds": context.video_duration,
        "voiceover": rel(root_path(args.output_wav)),
        "voiceover_video": rel(root_path(args.output_video)),
        "audio_dir": rel(root_path(args.audio_dir)),
        "audio_stem_template": args.audio_stem_template,
        "overrides": rel(root_path(args.overrides)),
        "cutting": {
            "source_cut_lead_seconds": args.cut_lead,
            "source_cut_tail_seconds": args.cut_tail,
            "clip_fade_in_seconds": args.clip_fade_in,
            "clip_fade_out_seconds": args.clip_fade_out,
            "min_gap_seconds": args.min_gap,
        },
        "placements": placements,
        "errors": errors,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--video", default=str(DEFAULT_VIDEO))
    parser.add_argument("--subtitle-report", default=str(SUBTITLE_REPORT))
    parser.add_argument("--audio-dir", default=str(audio_dir()))
    parser.add_argument("--audio-stem-template", default="{scene}")
    parser.add_argument("--provider", default="gemini")
    parser.add_argument("--scenes", default="", help="Comma-separated scene ids for focused placement tests.")
    parser.add_argument("--overrides", default=str(DEFAULT_OVERRIDES))
    parser.add_argument("--output-wav", default=str(DEFAULT_OUTPUT_WAV))
    parser.add_argument("--output-video", default=str(DEFAULT_OUTPUT_VIDEO))
    parser.add_argument("--report", default=str(DEFAULT_REPORT))
    parser.add_argument("--allow-missing-audio", action="store_true")
    parser.add_argument("--force-analysis", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--noise-db", default="-35dB")
    parser.add_argument("--min-silence", type=float, default=0.15)
    parser.add_argument("--min-speech", type=float, default=0.12)
    parser.add_argument("--utterance-gap", type=float, default=0.85)
    parser.add_argument("--cut-lead", type=float, default=0.08)
    parser.add_argument("--cut-tail", type=float, default=0.12)
    parser.add_argument("--clip-fade-in", type=float, default=0.015)
    parser.add_argument("--clip-fade-out", type=float, default=0.025)
    parser.add_argument("--min-gap", type=float, default=0.08)
    parser.add_argument("--event-delta", type=float, default=0.25)
    parser.add_argument("--duration-tolerance", type=float, default=0.15)
    parser.add_argument("--scene-overrun-tolerance", type=float, default=0.75)
    parser.add_argument("--no-shift-overlaps", action="store_true")
    args = parser.parse_args()

    if not shutil.which("ffmpeg") or not shutil.which("ffprobe"):
        print("ffmpeg and ffprobe are required", file=sys.stderr)
        return 1

    context = load_context(args)
    placements, errors = build_placements(args, context)
    report = report_data(args, context, placements, errors)
    write_json(root_path(args.report), report)
    if errors:
        print(f"voiceover placement failed; report written to {rel(root_path(args.report))}", file=sys.stderr)
        return 1
    if args.dry_run:
        print(rel(root_path(args.report)))
        return 0

    render_voiceover(placements, root_path(args.output_wav), context.video_duration, args)
    voiceover_duration = ffprobe_duration(root_path(args.output_wav))
    if abs(voiceover_duration - context.video_duration) > args.duration_tolerance:
        report["status"] = "failed"
        report["errors"].append(
            {
                "type": "voiceover-duration-mismatch",
                "voiceover_duration_seconds": voiceover_duration,
                "video_duration_seconds": context.video_duration,
            }
        )
        write_json(root_path(args.report), report)
        print(f"voiceover duration mismatch; report written to {rel(root_path(args.report))}", file=sys.stderr)
        return 1
    mux_video(root_path(args.video), root_path(args.output_wav), root_path(args.output_video))
    report["voiceover_duration_seconds"] = voiceover_duration
    report["voiceover_video_duration_seconds"] = ffprobe_duration(root_path(args.output_video))
    report["status"] = "ok"
    write_json(root_path(args.report), report)
    print(rel(root_path(args.output_wav)))
    print(rel(root_path(args.output_video)))
    print(rel(root_path(args.report)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
