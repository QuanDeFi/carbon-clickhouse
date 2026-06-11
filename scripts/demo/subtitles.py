#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
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
VOICEOVER_VIDEO = VIDEOS_DIR / "clickhouse-sink-tutorial-human-voiceover.mp4"
VOICEOVER_SUBTITLED_VIDEO = VIDEOS_DIR / "clickhouse-sink-tutorial-human-voiceover-subtitled.mp4"
VOICEOVER_PLACEMENT_REPORT = REVIEW_DIR / "audio/voiceover-placement-report.json"
MAX_SUBTITLE_LINE_CHARS = 42
MAX_SUBTITLE_CHARS = MAX_SUBTITLE_LINE_CHARS * 2
MAX_SUBTITLE_WORDS = 20
MAX_CUE_SECONDS = 7.0
MIN_CUE_SECONDS = 1.35
MIN_INTER_CUE_GAP_SECONDS = 0.12
MAX_INTER_CUE_GAP_SECONDS = 0.72
DEFAULT_SUBTITLE_WPM = 155.0
SCENE_SUBTITLE_LEAD_SECONDS = 0.8
SCENE_SUBTITLE_TAIL_SECONDS = 0.24
TERMINAL_SUBTITLE_SCENES = {
    "scene-03-jupiter-live",
    "scene-04-token-live",
}
SUBTITLE_LAYOUTS = {
    "default": {
        "line_chars": 42,
        "max_lines": 2,
        "max_words": 20,
        "style": "Default",
    },
    "terminal_right": {
        "line_chars": 32,
        "max_lines": 3,
        "max_words": 20,
        "style": "TerminalRight",
    },
}
WEAK_LINE_END_WORDS = {
    "a",
    "an",
    "and",
    "are",
    "as",
    "at",
    "by",
    "for",
    "from",
    "if",
    "in",
    "is",
    "of",
    "on",
    "or",
    "port",
    "so",
    "the",
    "to",
    "we",
    "with",
}
WEAK_LINE_START_WORDS = {
    "a",
    "an",
    "are",
    "as",
    "at",
    "by",
    "for",
    "from",
    "in",
    "is",
    "of",
    "on",
    "or",
    "to",
    "with",
}
PHRASE_START_WORDS = {
    "also",
    "and",
    "because",
    "but",
    "plus",
    "since",
    "so",
    "then",
    "to",
    "which",
    "while",
}


@dataclass
class Cue:
    index: int
    start: float
    end: float
    text: str
    scene: str
    layout: str


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


def word_count(text: str) -> int:
    return len(text.split())


def target_subtitle_wpm() -> float:
    raw = os.environ.get("DEMO_SUBTITLE_WPM", str(DEFAULT_SUBTITLE_WPM))
    try:
        value = float(raw)
    except ValueError:
        value = DEFAULT_SUBTITLE_WPM
    return max(90.0, min(180.0, value))


def scene_layout(scene: str) -> str:
    if terminal_right_subtitles_enabled() and scene in TERMINAL_SUBTITLE_SCENES:
        return "terminal_right"
    return "default"


def terminal_right_subtitles_enabled() -> bool:
    value = os.environ.get("DEMO_SUBTITLE_TERMINAL_RIGHT", "false")
    return value.strip().lower() in {"1", "true", "yes", "on"}


def layout_options(layout: str) -> dict:
    return SUBTITLE_LAYOUTS.get(layout, SUBTITLE_LAYOUTS["default"])


def render_lines_fit(text: str, layout: str = "default") -> bool:
    options = layout_options(layout)
    max_line_chars = options["line_chars"]
    max_lines = options["max_lines"]
    words = text.split()
    if len(text) <= max_line_chars:
        return True
    if max_lines == 2:
        for split_at in range(1, len(words)):
            first = " ".join(words[:split_at])
            second = " ".join(words[split_at:])
            if len(first) <= max_line_chars and len(second) <= max_line_chars:
                return True
    else:
        return len(textwrap.wrap(text, width=max_line_chars)) <= max_lines
    return False


def caption_fits(text: str, layout: str = "default") -> bool:
    options = layout_options(layout)
    return (
        len(text) <= options["line_chars"] * options["max_lines"]
        and word_count(text) <= options["max_words"]
        and render_lines_fit(text, layout)
    )


def split_words(text: str, layout: str = "default") -> list[str]:
    chunks: list[str] = []
    current: list[str] = []
    for word in text.split():
        candidate = " ".join(current + [word])
        if current and not caption_fits(candidate, layout):
            chunks.append(" ".join(current))
            current = [word]
        else:
            current.append(word)
    if current:
        chunks.append(" ".join(current))
    return chunks


def normalized_word(word: str) -> str:
    return re.sub(r"[^A-Za-z0-9]+", "", word).lower()


def split_candidate_penalty(left: str, right: str, total_chars: int) -> int:
    left_words = left.split()
    right_words = right.split()
    if not left_words or not right_words:
        return 10_000
    left_last = normalized_word(left_words[-1])
    right_first = normalized_word(right_words[0])
    penalty = abs(len(left) - min(total_chars * 0.55, 70))
    if left_last in WEAK_LINE_END_WORDS:
        penalty += 90
    if right_first in WEAK_LINE_START_WORDS:
        penalty += 12
    if right_first in PHRASE_START_WORDS:
        penalty -= 25
    if len(left) < 24:
        penalty += 70
    if len(right) < 24:
        penalty += 80
    if len(left_words) > 1 and normalized_word(left_words[-2]) in {"and", "or"}:
        penalty += 80
    if re.search(r"[,;:]$", left_words[-1]):
        penalty -= 35
    if left_last == "port" and re.match(r"^\d", right_words[0]):
        penalty += 200
    return int(penalty)


def natural_split_long_sentence(sentence: str, layout: str = "default") -> list[str] | None:
    words = sentence.split()
    if len(words) < 4:
        return None
    candidates: list[tuple[int, int, str, str]] = []
    total_chars = len(sentence)
    for split_at in range(1, len(words)):
        left = " ".join(words[:split_at]).strip()
        right = " ".join(words[split_at:]).strip()
        if not caption_fits(left, layout):
            continue
        left_last = words[split_at - 1]
        right_first = normalized_word(words[split_at])
        natural_boundary = (
            bool(re.search(r"[,;:]$", left_last))
            or right_first in PHRASE_START_WORDS
            or split_candidate_penalty(left, right, total_chars) < 45
        )
        if not natural_boundary:
            continue
        candidates.append((split_candidate_penalty(left, right, total_chars), split_at, left, right))
    if not candidates:
        return None
    _score, _split_at, left, right = sorted(candidates, key=lambda item: item[0])[0]
    return [left, *split_long_sentence(right, layout)]


def split_long_sentence(sentence: str, layout: str = "default") -> list[str]:
    if caption_fits(sentence, layout):
        return [sentence]
    natural = natural_split_long_sentence(sentence, layout)
    if natural:
        return natural
    return split_words(sentence, layout)


def sentence_chunks(text: str, layout: str = "default") -> list[str]:
    chunks: list[str] = []
    for paragraph in [part.strip() for part in text.split("\n\n") if part.strip()]:
        sentences = re.split(r"(?<=[.!?])\s+", paragraph.replace("\n", " ").strip())
        current = ""
        for sentence in [part.strip() for part in sentences if part.strip()]:
            if not caption_fits(sentence, layout):
                if current:
                    chunks.append(current)
                    current = ""
                chunks.extend(split_long_sentence(sentence, layout))
                continue
            candidate = f"{current} {sentence}".strip()
            if current and not caption_fits(candidate, layout):
                chunks.append(current)
                current = sentence
            else:
                current = candidate
        if current:
            chunks.append(current)
    return chunks


def balanced_caption_lines(text: str, layout: str = "default") -> list[str]:
    options = layout_options(layout)
    max_line_chars = options["line_chars"]
    max_lines = options["max_lines"]
    words = text.split()
    if len(text) <= max_line_chars or len(words) <= 1:
        return [text]

    best: tuple[int, list[str]] | None = None
    if max_lines == 2:
        for split_at in range(1, len(words)):
            first = " ".join(words[:split_at])
            second = " ".join(words[split_at:])
            if len(first) > max_line_chars or len(second) > max_line_chars:
                continue
            first_last = normalized_word(words[split_at - 1])
            second_first = normalized_word(words[split_at])
            score = abs(len(first) - len(second))
            if first_last in WEAK_LINE_END_WORDS:
                score += 80
            if second_first in WEAK_LINE_START_WORDS:
                score += 12
            if first_last == "port" and re.match(r"^\d", words[split_at]):
                score += 200
            if re.search(r"[,;:]$", words[split_at - 1]):
                score -= 20
            if best is None or score < best[0]:
                best = (score, [first, second])
        if best is not None:
            return best[1]

    return textwrap.wrap(text, width=max_line_chars)[:max_lines]


def wrap_caption(text: str, layout: str = "default") -> str:
    return "\n".join(balanced_caption_lines(text, layout))


def cue_durations(weights: list[int], available: float) -> list[float]:
    if not weights:
        return []
    total = sum(weights)
    durations = [available * weight / total for weight in weights]
    floor = min(1.0, (available / len(weights)) * 0.8)
    durations = [max(floor, value) for value in durations]
    excess = sum(durations) - available
    if excess > 0:
        adjustable = sum(max(0.0, value - floor) for value in durations)
        if adjustable > 0:
            durations = [
                value - excess * max(0.0, value - floor) / adjustable
                for value in durations
            ]
        else:
            scale = available / sum(durations)
            durations = [value * scale for value in durations]
    elif excess < 0:
        remaining = -excess
        durations = [
            value + remaining * weight / total
            for value, weight in zip(durations, weights)
        ]
    for _ in range(4):
        over = sum(max(0.0, value - MAX_CUE_SECONDS) for value in durations)
        if over <= 0:
            break
        durations = [min(value, MAX_CUE_SECONDS) for value in durations]
        capacity = sum(max(0.0, MAX_CUE_SECONDS - value) for value in durations)
        if capacity <= 0:
            break
        durations = [
            value + over * max(0.0, MAX_CUE_SECONDS - value) / capacity
            for value in durations
        ]
    return durations


def natural_cue_duration(text: str) -> float:
    words = max(1, word_count(text))
    chars = max(1, len(text.replace("\n", " ")))
    speech_seconds = words * 60.0 / target_subtitle_wpm()
    reading_seconds = chars / 18.0
    sentence_pause = len(re.findall(r"[.!?](?:\s|$)", text)) * 0.20
    phrase_pause = len(re.findall(r"[,;:]", text)) * 0.08
    return min(MAX_CUE_SECONDS, max(MIN_CUE_SECONDS, speech_seconds, reading_seconds) + sentence_pause + phrase_pause)


def scene_caption_chunks(scene: str) -> tuple[str, list[str]]:
    script = ROOT / "docs/tutorial-video" / f"{scene}.md"
    layout = scene_layout(scene)
    return layout, sentence_chunks(clean_markdown(script), layout)


def natural_scene_required_seconds(chunks: list[str], scene_duration: float) -> float:
    if not chunks:
        return 0.0
    lead = min(SCENE_SUBTITLE_LEAD_SECONDS, scene_duration * 0.08)
    tail = min(SCENE_SUBTITLE_TAIL_SECONDS, scene_duration * 0.08)
    return lead + tail + sum(natural_cue_duration(chunk) for chunk in chunks) + (len(chunks) - 1) * MIN_INTER_CUE_GAP_SECONDS


def make_scene_cues(scene: str, scene_start: float, scene_duration: float, first_index: int) -> list[Cue]:
    layout, chunks = scene_caption_chunks(scene)
    if not chunks:
        return []
    lead = min(SCENE_SUBTITLE_LEAD_SECONDS, scene_duration * 0.08)
    tail = min(SCENE_SUBTITLE_TAIL_SECONDS, scene_duration * 0.08)
    natural_durations = [natural_cue_duration(chunk) for chunk in chunks]
    required = lead + tail + sum(natural_durations) + (len(chunks) - 1) * MIN_INTER_CUE_GAP_SECONDS
    if required <= scene_duration:
        durations = natural_durations
        leftover = scene_duration - required
        if len(chunks) > 1:
            gap = MIN_INTER_CUE_GAP_SECONDS + min(
                MAX_INTER_CUE_GAP_SECONDS - MIN_INTER_CUE_GAP_SECONDS,
                leftover / (len(chunks) - 1),
            )
        else:
            gap = 0.0
    else:
        available = max(0.5, scene_duration - lead - tail)
        weights = [max(8, len(chunk)) for chunk in chunks]
        durations = cue_durations(weights, available)
        gap = 0.0
    cursor = scene_start + lead
    cues: list[Cue] = []
    for offset, (chunk, cue_duration) in enumerate(zip(chunks, durations), start=0):
        start = cursor
        end = min(scene_start + scene_duration - tail, cursor + cue_duration)
        cues.append(Cue(first_index + offset, start, end, wrap_caption(chunk, layout), scene, layout))
        cursor = end + gap
    return cues


def placement_text(placement: dict) -> str:
    text = str(placement.get("aligned_text") or placement.get("text") or "").strip()
    return " ".join(text.split())


def make_span_cues(
    *,
    scene: str,
    text: str,
    start: float,
    end: float,
    first_index: int,
) -> list[Cue]:
    layout = scene_layout(scene)
    chunks = sentence_chunks(text, layout)
    if not chunks or end <= start:
        return []

    span = end - start
    gap = MIN_INTER_CUE_GAP_SECONDS if len(chunks) > 1 and span / len(chunks) >= 1.15 else 0.0
    available = max(0.05, span - gap * (len(chunks) - 1))
    weights = [max(8, len(chunk.replace("\n", " "))) for chunk in chunks]
    total_weight = max(1, sum(weights))
    durations = [available * weight / total_weight for weight in weights]

    cues: list[Cue] = []
    cursor = start
    for offset, (chunk, cue_duration) in enumerate(zip(chunks, durations), start=0):
        cue_start = cursor
        cue_end = end if offset == len(chunks) - 1 else min(end, cursor + cue_duration)
        cues.append(Cue(first_index + offset, cue_start, cue_end, wrap_caption(chunk, layout), scene, layout))
        cursor = cue_end + gap
    return cues


def build_voiceover_cues() -> tuple[list[Cue], dict]:
    if not VOICEOVER_PLACEMENT_REPORT.is_file():
        raise FileNotFoundError(VOICEOVER_PLACEMENT_REPORT)
    if not (SUBTITLE_DIR / "subtitle-report.json").is_file():
        raise FileNotFoundError(SUBTITLE_DIR / "subtitle-report.json")

    base_report = json.loads((SUBTITLE_DIR / "subtitle-report.json").read_text())
    placement_report = json.loads(VOICEOVER_PLACEMENT_REPORT.read_text())
    placements = sorted(
        placement_report.get("placements", []),
        key=lambda item: (float(item.get("target_timeline_start_seconds", 0)), str(item.get("scene", ""))),
    )

    cues: list[Cue] = []
    for placement in placements:
        text = placement_text(placement)
        if not text:
            continue
        scene = str(placement["scene"])
        start = float(placement["target_timeline_start_seconds"])
        end = float(placement["target_timeline_end_seconds"])
        cues.extend(make_span_cues(scene=scene, text=text, start=start, end=end, first_index=len(cues) + 1))

    scenes = []
    for base_scene in base_report.get("scenes", []):
        scene = dict(base_scene)
        scene_cues = [cue for cue in cues if cue.scene == scene["scene"]]
        cue_words = sum(word_count(cue.text.replace("\n", " ")) for cue in scene_cues)
        cue_seconds = sum(cue.end - cue.start for cue in scene_cues)
        scene["cue_count"] = len(scene_cues)
        scene["subtitle_source"] = "voiceover-placement"
        scene["caption_words_per_minute"] = round((cue_words / cue_seconds) * 60, 1) if cue_seconds else 0
        scene["natural_pacing_fits_scene"] = True
        scenes.append(scene)

    report = {
        "source": "voiceover-placement",
        "source_report": str(VOICEOVER_PLACEMENT_REPORT.relative_to(ROOT)),
        "voiceover_video": str(VOICEOVER_VIDEO.relative_to(ROOT)),
        "scenes": scenes,
        "total_cues": len(cues),
        "total_duration_seconds": base_report.get("total_duration_seconds", 0),
    }
    return cues, report


def srt_time(seconds: float) -> str:
    millis = int(round(seconds * 1000))
    hours, remainder = divmod(millis, 3_600_000)
    minutes, remainder = divmod(remainder, 60_000)
    secs, millis = divmod(remainder, 1000)
    return f"{hours:02}:{minutes:02}:{secs:02},{millis:03}"


def vtt_time(seconds: float) -> str:
    return srt_time(seconds).replace(",", ".")


def ass_time(seconds: float) -> str:
    centis = int(round(seconds * 100))
    hours, remainder = divmod(centis, 360_000)
    minutes, remainder = divmod(remainder, 6_000)
    secs, centis = divmod(remainder, 100)
    return f"{hours}:{minutes:02}:{secs:02}.{centis:02}"


def ass_text(text: str) -> str:
    return text.replace("\\", "\\\\").replace("{", r"\{").replace("}", r"\}").replace("\n", r"\N")


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


def write_ass(cues: list[Cue], path: Path) -> None:
    lines = [
        "[Script Info]",
        "ScriptType: v4.00+",
        "PlayResX: 1920",
        "PlayResY: 1080",
        "WrapStyle: 2",
        "ScaledBorderAndShadow: yes",
        "",
        "[V4+ Styles]",
        "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding",
        "Style: Default,DejaVu Sans,28,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,3,0,2,180,180,18,1",
        "Style: TerminalRight,DejaVu Sans,27,&H00FFFFFF,&H00FFFFFF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,3,0,6,1160,88,0,1",
        "",
        "[Events]",
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text",
    ]
    for cue in cues:
        style = layout_options(cue.layout)["style"]
        lines.append(f"Dialogue: 0,{ass_time(cue.start)},{ass_time(cue.end)},{style},,0,0,0,,{ass_text(cue.text)}")
    path.write_text("\n".join(lines) + "\n")


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
        layout, chunks = scene_caption_chunks(scene)
        cue_words = sum(word_count(cue.text.replace("\n", " ")) for cue in scene_cues)
        cue_seconds = sum(cue.end - cue.start for cue in scene_cues)
        required_seconds = natural_scene_required_seconds(chunks, scene_duration)
        report["scenes"].append(
            {
                "scene": scene,
                "start_seconds": round(offset, 3),
                "duration_seconds": round(scene_duration, 3),
                "cue_count": len(scene_cues),
                "layout": layout,
                "natural_required_seconds": round(required_seconds, 3),
                "natural_pacing_fits_scene": required_seconds <= scene_duration + 0.001,
                "caption_words_per_minute": round((cue_words / cue_seconds) * 60, 1) if cue_seconds else 0,
            }
        )
        offset += scene_duration
    report["total_cues"] = len(cues)
    report["total_duration_seconds"] = round(offset, 3)
    return cues, report


def ffmpeg_subtitle_path(path: Path) -> str:
    # FFmpeg subtitles filter needs ':' and apostrophes escaped in filenames.
    return str(path.resolve()).replace("\\", "\\\\").replace(":", "\\:").replace("'", "\\'")


def burn_subtitles(ass: Path, output: Path, *, source_video: Path, keep_audio: bool) -> None:
    if not source_video.is_file():
        raise FileNotFoundError(source_video)
    command = [
        "ffmpeg",
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        str(source_video),
        "-vf",
        f"ass='{ffmpeg_subtitle_path(ass)}'",
    ]
    if keep_audio:
        command.extend(["-c:a", "copy"])
    else:
        command.append("-an")
    command.extend(
        [
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-pix_fmt",
            "yuv420p",
            str(output),
        ]
    )
    subprocess.run(
        command,
        cwd=ROOT,
        check=True,
    )


def generate(*, burn: bool, source: str = "scene") -> dict:
    SUBTITLE_DIR.mkdir(parents=True, exist_ok=True)
    if source == "voiceover":
        cues, report = build_voiceover_cues()
        stem = "clickhouse-sink-tutorial-human-voiceover"
        source_video = VOICEOVER_VIDEO
        subtitled_video = VOICEOVER_SUBTITLED_VIDEO
        report_path = SUBTITLE_DIR / "voiceover-subtitle-report.json"
        keep_audio = True
    else:
        cues, report = build_cues()
        stem = "clickhouse-sink-tutorial-human-no-audio"
        source_video = FINAL_VIDEO
        subtitled_video = SUBTITLED_VIDEO
        report_path = SUBTITLE_DIR / "subtitle-report.json"
        keep_audio = False

    srt = SUBTITLE_DIR / f"{stem}.srt"
    vtt = SUBTITLE_DIR / f"{stem}.vtt"
    ass = SUBTITLE_DIR / f"{stem}.ass"
    write_srt(cues, srt)
    write_vtt(cues, vtt)
    write_ass(cues, ass)
    durations = [cue.end - cue.start for cue in cues]
    line_lengths = [
        len(line)
        for cue in cues
        for line in cue.text.splitlines()
    ]
    report.update(
        {
            "srt": str(srt.relative_to(ROOT)),
            "vtt": str(vtt.relative_to(ROOT)),
            "ass": str(ass.relative_to(ROOT)),
            "source_video": str(source_video.relative_to(ROOT)),
            "subtitle_style": {
                "placement": "centered lower subtitle",
                "terminal_placement": "right side, vertically centered when DEMO_SUBTITLE_TERMINAL_RIGHT=true",
                "terminal_right_enabled": terminal_right_subtitles_enabled(),
                "target_words_per_minute": target_subtitle_wpm(),
                "max_lines": 2,
                "terminal_max_lines": 3,
                "max_line_chars": MAX_SUBTITLE_LINE_CHARS,
                "terminal_max_line_chars": SUBTITLE_LAYOUTS["terminal_right"]["line_chars"],
                "font": "DejaVu Sans with black outline and no shadow",
            },
            "cue_duration_seconds": {
                "min": round(min(durations), 3) if durations else 0,
                "max": round(max(durations), 3) if durations else 0,
                "average": round(sum(durations) / len(durations), 3) if durations else 0,
            },
            "max_rendered_line_chars": max(line_lengths) if line_lengths else 0,
        }
    )
    if burn:
        burn_subtitles(ass, subtitled_video, source_video=source_video, keep_audio=keep_audio)
        report["subtitled_video"] = str(subtitled_video.relative_to(ROOT))
        report["subtitled_video_duration_seconds"] = round(duration(subtitled_video), 3)
    report_path.write_text(json.dumps(report, indent=2))
    report["report"] = str(report_path.relative_to(ROOT))
    return report


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--no-burn", action="store_true", help="Only write SRT/VTT sidecars.")
    parser.add_argument("--source", choices=["scene", "voiceover"], default="scene")
    parser.add_argument("--voiceover", action="store_true", help="Shortcut for --source voiceover.")
    args = parser.parse_args()
    source = "voiceover" if args.voiceover else args.source
    report = generate(burn=not args.no_burn, source=source)
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
