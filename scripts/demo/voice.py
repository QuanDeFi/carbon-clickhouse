#!/usr/bin/env python3
from __future__ import annotations

import argparse
import base64
import io
import json
import os
import re
import shutil
import subprocess
import sys
import time
import wave
from pathlib import Path
from typing import Any

import requests


ROOT = Path(__file__).resolve().parents[2]

ELEVENLABS_VOICES_URL = "https://api.elevenlabs.io/v2/voices"
ELEVENLABS_TTS_URL_TEMPLATE = "https://api.elevenlabs.io/v1/text-to-speech/{voice_id}"
DEFAULT_ELEVENLABS_MODEL_ID = "eleven_multilingual_v2"
DEFAULT_ELEVENLABS_OUTPUT_FORMAT = "mp3_44100_128"

GEMINI_TTS_URL_TEMPLATE = "https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent"
DEFAULT_GEMINI_MODEL = "gemini-3.1-flash-tts-preview"
DEFAULT_GEMINI_VOICE = "Iapetus"
DEFAULT_GEMINI_SAMPLE_RATE = 24000
DEFAULT_GEMINI_SAMPLE_WIDTH = 2
DEFAULT_GEMINI_PROMPT_STYLE = "software-tutorial"

DEFAULT_ALIBABA_DASHSCOPE_MODEL = "cosyvoice-v3-plus"
DEFAULT_ALIBABA_DASHSCOPE_VOICE = "longanyang"
DEFAULT_ALIBABA_DASHSCOPE_FORMAT = "mp3"
DEFAULT_ALIBABA_DASHSCOPE_MAX_CHARS = 20000
DEFAULT_DASHSCOPE_BASE_WEBSOCKET_API_URL = "wss://dashscope-intl.aliyuncs.com/api-ws/v1/inference"

SUPPORTED_AUDIO_SUFFIXES = (".mp3", ".wav", ".pcm")


def load_env() -> None:
    initial = set(os.environ)

    def load_file(path: Path, *, override_loaded: bool = False) -> None:
        if not path.is_file():
            return
        for raw in path.read_text(errors="ignore").splitlines():
            line = raw.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, value = line.split("=", 1)
            key = key.strip()
            if not key.replace("_", "").isalnum() or key[0].isdigit():
                continue
            if key not in os.environ or (override_loaded and key not in initial):
                os.environ[key] = value.strip().strip('"').strip("'")

    for path in (ROOT / ".env.demo.local", ROOT / "scripts/demo/.env.demo.local"):
        if path.is_file():
            load_file(path)
            break
    load_file(ROOT / "demo-artifacts/display.env", override_loaded=True)


def present(key: str) -> bool:
    value = os.environ.get(key, "")
    return bool(value and not value.startswith("<"))


def env_bool(key: str, default: bool = False) -> bool:
    value = os.environ.get(key)
    if value is None:
        return default
    return value.strip().lower() in {"1", "true", "yes", "on"}


def env_int(key: str, default: int) -> int:
    value = os.environ.get(key)
    if not value:
        return default
    try:
        return int(value)
    except ValueError:
        return default


def env_value(key: str, default: str = "") -> str:
    value = os.environ.get(key, "")
    if value and not value.startswith("<"):
        return value
    return default


def tts_provider() -> str:
    return os.environ.get("TTS_PROVIDER", "elevenlabs").strip().lower().replace("_", "-")


def audio_dir() -> Path:
    path = ROOT / os.environ.get("DEMO_AUDIO_DIR", "demo-artifacts/audio")
    path.mkdir(parents=True, exist_ok=True)
    return path


def clean_extension(extension: str) -> str:
    extension = extension.lower().lstrip(".")
    if not extension:
        return "mp3"
    return extension


def audio_output_path(name: str, extension: str) -> Path:
    extension = clean_extension(extension)
    output = audio_dir() / f"{name}.{extension}"
    for suffix in SUPPORTED_AUDIO_SUFFIXES:
        sibling = output.with_suffix(suffix)
        if sibling != output and sibling.exists():
            sibling.unlink()
    return output


def write_audio_metadata(output: Path, metadata: dict[str, Any]) -> None:
    data = {
        **metadata,
        "output": str(output.relative_to(ROOT)),
        "bytes": output.stat().st_size,
        "generated_at": time.time(),
    }
    output.with_suffix(".json").write_text(json.dumps(data, indent=2) + "\n")


def write_audio_file(name: str, audio: bytes, extension: str, metadata: dict[str, Any]) -> Path:
    output = audio_output_path(name, extension)
    output.write_bytes(audio)
    write_audio_metadata(output, metadata)
    print(f"audio_written: {output.relative_to(ROOT)}")
    return output


def request_error(response: requests.Response) -> str:
    try:
        body = response.json()
    except Exception:
        body = {}

    if isinstance(body, dict):
        error = body.get("error")
        if isinstance(error, dict):
            status = error.get("status")
            message = error.get("message")
            if status and message:
                return f"{status}: {message}"
            if message:
                return str(message)
            if status:
                return str(status)

        detail = body.get("detail")
        if isinstance(detail, dict):
            status = detail.get("status")
            message = detail.get("message")
            if status and message:
                return f"{status}: {message}"
            if status:
                return str(status)
            if message:
                return str(message)
        if isinstance(detail, str):
            return detail
        for key in ("message", "Message", "error", "Code", "ErrMsg"):
            if body.get(key):
                return str(body[key])

    return response.text[:240]


def status_line(key: str, configured: bool, default: str | None = None) -> None:
    if configured:
        print(f"{key}: present")
    elif default is not None:
        print(f"{key}: defaulted to {default}")
    else:
        print(f"{key}: missing")


def status() -> int:
    provider = tts_provider()
    print(f"TTS_PROVIDER: {provider}")
    if provider == "elevenlabs":
        status_line("ELEVENLABS_API_KEY", present("ELEVENLABS_API_KEY"))
        status_line("ELEVENLABS_VOICE_ID", present("ELEVENLABS_VOICE_ID"))
        status_line("ELEVENLABS_MODEL_ID", present("ELEVENLABS_MODEL_ID"), DEFAULT_ELEVENLABS_MODEL_ID)
        status_line("ELEVENLABS_OUTPUT_FORMAT", present("ELEVENLABS_OUTPUT_FORMAT"), DEFAULT_ELEVENLABS_OUTPUT_FORMAT)
        print(
            "ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK: "
            + ("present/enabled" if env_bool("ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK") else "missing/defaulted to false")
        )
    elif provider == "gemini":
        status_line("GEMINI_API_KEY or GOOGLE_API_KEY", present("GEMINI_API_KEY") or present("GOOGLE_API_KEY"))
        status_line("GEMINI_TTS_MODEL", present("GEMINI_TTS_MODEL"), DEFAULT_GEMINI_MODEL)
        status_line("GEMINI_TTS_VOICE", present("GEMINI_TTS_VOICE"), DEFAULT_GEMINI_VOICE)
        status_line("GEMINI_TTS_SAMPLE_RATE", present("GEMINI_TTS_SAMPLE_RATE"), str(DEFAULT_GEMINI_SAMPLE_RATE))
        status_line("GEMINI_TTS_TEMPERATURE", present("GEMINI_TTS_TEMPERATURE"), "1")
        status_line("GEMINI_TTS_PROMPT_STYLE", present("GEMINI_TTS_PROMPT_STYLE"), DEFAULT_GEMINI_PROMPT_STYLE)
        status_line(
            "GEMINI_TTS_PROMPT_PREFIX or GEMINI_TTS_PROMPT_PREFIX_FILE",
            present("GEMINI_TTS_PROMPT_PREFIX") or present("GEMINI_TTS_PROMPT_PREFIX_FILE"),
        )
    elif provider == "alibaba":
        status_line("DASHSCOPE_API_KEY or ALIBABA_DASHSCOPE_API_KEY", bool(alibaba_dashscope_api_key()))
        status_line("ALIBABA_DASHSCOPE_MODEL", present("ALIBABA_DASHSCOPE_MODEL"), DEFAULT_ALIBABA_DASHSCOPE_MODEL)
        status_line("ALIBABA_DASHSCOPE_VOICE", present("ALIBABA_DASHSCOPE_VOICE"), DEFAULT_ALIBABA_DASHSCOPE_VOICE)
        status_line("ALIBABA_DASHSCOPE_AUDIO_FORMAT", present("ALIBABA_DASHSCOPE_AUDIO_FORMAT"), "default MP3")
        status_line(
            "DASHSCOPE_BASE_WEBSOCKET_API_URL",
            present("DASHSCOPE_BASE_WEBSOCKET_API_URL") or present("ALIBABA_DASHSCOPE_BASE_WEBSOCKET_API_URL"),
            DEFAULT_DASHSCOPE_BASE_WEBSOCKET_API_URL,
        )
    else:
        print(f"Unsupported TTS_PROVIDER: {provider}", file=sys.stderr)
        return 1
    return 0


def check() -> int:
    provider = tts_provider()
    if provider == "elevenlabs":
        if not present("ELEVENLABS_API_KEY"):
            print("Missing ElevenLabs env: ELEVENLABS_API_KEY", file=sys.stderr)
            return 1
        if not present("ELEVENLABS_VOICE_ID") and env_bool("ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK"):
            print("Missing ElevenLabs env: ELEVENLABS_VOICE_ID", file=sys.stderr)
            return 1
        return 0
    if provider == "gemini":
        if present("GEMINI_API_KEY") or present("GOOGLE_API_KEY"):
            return 0
        print("Missing Gemini env: GEMINI_API_KEY or GOOGLE_API_KEY", file=sys.stderr)
        return 1
    if provider == "alibaba":
        missing = []
        if not alibaba_dashscope_api_key():
            missing.append("DASHSCOPE_API_KEY or ALIBABA_DASHSCOPE_API_KEY")
        if not dashscope_sdk_available():
            missing.append("dashscope Python package")
        if missing:
            print("Missing Alibaba DashScope setup: " + ", ".join(missing), file=sys.stderr)
            return 1
        try:
            alibaba_dashscope_audio_format()
            alibaba_dashscope_extension()
        except RuntimeError as exc:
            print(str(exc), file=sys.stderr)
            return 1
        return 0
    print(f"Unsupported TTS_PROVIDER: {provider}", file=sys.stderr)
    return 1


def require_elevenlabs_api_key() -> bool:
    if not present("ELEVENLABS_API_KEY"):
        print("Missing ElevenLabs env: ELEVENLABS_API_KEY", file=sys.stderr)
        return False
    return True


def fetch_voices(params: dict[str, Any]) -> list[dict[str, Any]]:
    response = requests.get(
        ELEVENLABS_VOICES_URL,
        params=params,
        headers={"xi-api-key": os.environ["ELEVENLABS_API_KEY"]},
        timeout=30,
    )
    if response.status_code >= 400:
        raise RuntimeError(f"ElevenLabs voices request failed with status {response.status_code}: {request_error(response)}")
    data = response.json()
    voices = data.get("voices", [])
    if not isinstance(voices, list):
        raise RuntimeError("ElevenLabs voices response did not include a voices array")
    return [voice for voice in voices if isinstance(voice, dict)]


def voice_tiers_include_free(voice: dict[str, Any]) -> bool:
    tiers = voice.get("available_for_tiers") or []
    if not isinstance(tiers, list):
        return False
    return any("free" in str(tier).lower() for tier in tiers)


def voice_is_free_usable(voice: dict[str, Any]) -> bool:
    category = str(voice.get("category") or "").lower()
    sharing = voice.get("sharing") or {}
    free_users_allowed = isinstance(sharing, dict) and sharing.get("free_users_allowed") is True
    return category in {"default", "premade"} or free_users_allowed or voice_tiers_include_free(voice)


def voice_label(voice: dict[str, Any]) -> str:
    name = str(voice.get("name") or "unnamed")
    category = str(voice.get("category") or "unknown")
    return f"{name} (category={category})"


def free_voice_candidates() -> list[dict[str, Any]]:
    default_voices = [voice for voice in fetch_voices({"page_size": 100, "voice_type": "default"}) if voice_is_free_usable(voice)]
    if default_voices:
        return default_voices
    return [voice for voice in fetch_voices({"page_size": 100}) if voice_is_free_usable(voice)]


def discover_free_voice() -> dict[str, Any] | None:
    voices = free_voice_candidates()
    return voices[0] if voices else None


def list_voices(include_all: bool) -> int:
    if tts_provider() != "elevenlabs":
        print("Voice listing is currently implemented only for the ElevenLabs provider.", file=sys.stderr)
        return 1
    if not require_elevenlabs_api_key():
        return 1
    try:
        voices = fetch_voices({"page_size": 100} if include_all else {"page_size": 100, "voice_type": "default"})
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    print("Voice IDs are intentionally not printed. Use `select-free --write-env` to update .env.demo.local.")
    print("Name\tCategory\tFree usable")
    for voice in voices:
        print(f"{voice.get('name') or 'unnamed'}\t{voice.get('category') or 'unknown'}\t{'yes' if voice_is_free_usable(voice) else 'no'}")
    return 0


def update_env_file(key: str, value: str) -> Path:
    path = ROOT / ".env.demo.local"
    lines: list[str] = []
    if path.is_file():
        lines = path.read_text(errors="ignore").splitlines()
    updated = False
    output: list[str] = []
    for line in lines:
        stripped = line.strip()
        if stripped and not stripped.startswith("#") and "=" in stripped:
            existing_key = stripped.split("=", 1)[0].strip()
            if existing_key == key:
                output.append(f"{key}={value}")
                updated = True
                continue
        output.append(line)
    if not updated:
        output.append(f"{key}={value}")
    path.write_text("\n".join(output).rstrip() + "\n")
    path.chmod(0o600)
    return path


def select_free(write_env: bool) -> int:
    if tts_provider() != "elevenlabs":
        print("Free voice selection is implemented only for the ElevenLabs provider.", file=sys.stderr)
        return 1
    if not require_elevenlabs_api_key():
        return 1
    try:
        voice = discover_free_voice()
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    if not voice or not voice.get("voice_id"):
        print("No free-usable/default ElevenLabs voice was returned for this account.", file=sys.stderr)
        return 1
    print(f"selected_voice: {voice_label(voice)}")
    if write_env:
        update_env_file("ELEVENLABS_VOICE_ID", str(voice["voice_id"]))
        print("ELEVENLABS_VOICE_ID: updated in .env.demo.local")
    else:
        print("Run with --write-env to store this voice in .env.demo.local without printing the voice ID.")
    return 0


def scene_text(scene: str) -> str:
    path = ROOT / "docs/tutorial-video" / f"{scene}.md"
    if not path.is_file():
        raise FileNotFoundError(path)
    lines = []
    for line in path.read_text().splitlines():
        if line.startswith("# "):
            continue
        lines.append(line)
    return "\n".join(lines).strip()


def elevenlabs_audio_extension(output_format: str) -> str:
    if output_format.lower().startswith("mp3"):
        return "mp3"
    if output_format.lower().startswith("pcm"):
        return "pcm"
    return "mp3"


def write_elevenlabs_audio(name: str, text: str, voice: dict[str, Any], voice_source: str) -> tuple[bool, int, str]:
    voice_id = str(voice["voice_id"])
    model_id = os.environ.get("ELEVENLABS_MODEL_ID", DEFAULT_ELEVENLABS_MODEL_ID)
    output_format = os.environ.get("ELEVENLABS_OUTPUT_FORMAT", DEFAULT_ELEVENLABS_OUTPUT_FORMAT)
    url = ELEVENLABS_TTS_URL_TEMPLATE.format(voice_id=voice_id)
    response = requests.post(
        url,
        params={"output_format": output_format},
        headers={
            "xi-api-key": os.environ["ELEVENLABS_API_KEY"],
            "accept": "audio/mpeg",
            "content-type": "application/json",
        },
        json={
            "text": text,
            "model_id": model_id,
        },
        timeout=120,
    )
    if response.status_code >= 400:
        return False, response.status_code, request_error(response)

    write_audio_file(
        name,
        response.content,
        elevenlabs_audio_extension(output_format),
        {
            "name": name,
            "provider": "elevenlabs",
            "model_id": model_id,
            "output_format": output_format,
            "voice_source": voice_source,
            "voice_name": voice.get("name"),
            "voice_category": voice.get("category"),
        },
    )
    return True, response.status_code, ""


def configured_voice() -> dict[str, Any] | None:
    if not present("ELEVENLABS_VOICE_ID"):
        return None
    return {
        "voice_id": os.environ["ELEVENLABS_VOICE_ID"],
        "name": "configured",
        "category": "configured",
    }


def generate_elevenlabs_audio(name: str, text: str) -> int:
    missing = [key for key in ("ELEVENLABS_API_KEY",) if not present(key)]
    if missing:
        print("Missing ElevenLabs env: " + ", ".join(missing), file=sys.stderr)
        return 1

    attempts: list[tuple[dict[str, Any], str]] = []
    configured = configured_voice()
    if configured:
        attempts.append((configured, "configured"))
    elif env_bool("ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK"):
        print("Missing ElevenLabs env: ELEVENLABS_VOICE_ID", file=sys.stderr)
        return 1

    if not env_bool("ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK"):
        try:
            fallback_voices = free_voice_candidates()
        except RuntimeError as exc:
            if not attempts:
                print(str(exc), file=sys.stderr)
                return 1
            fallback_voices = []
        for fallback_voice in fallback_voices:
            if fallback_voice and all(fallback_voice.get("voice_id") != voice.get("voice_id") for voice, _ in attempts):
                attempts.append((fallback_voice, "auto-free-default"))

    if not attempts:
        print("Missing ElevenLabs env: ELEVENLABS_VOICE_ID and no free-usable voice could be discovered.", file=sys.stderr)
        return 1

    last_status = 0
    last_error = ""
    for voice, source in attempts:
        ok, status_code, error = write_elevenlabs_audio(name, text, voice, source)
        if ok:
            if source == "auto-free-default":
                print(f"voice_selected: {voice_label(voice)}")
                if env_bool("ELEVENLABS_AUTO_WRITE_VOICE_ID"):
                    update_env_file("ELEVENLABS_VOICE_ID", str(voice["voice_id"]))
                    print("ELEVENLABS_VOICE_ID: updated in .env.demo.local")
            return 0
        last_status = status_code
        last_error = error
        if source == "configured" and status_code == 402 and "paid_plan_required" in error:
            print("Configured ElevenLabs voice is not available on this account tier; trying a default/free-usable voice.", file=sys.stderr)
            continue
        if source == "configured" and len(attempts) > 1:
            print("Configured ElevenLabs voice failed; trying a default/free-usable voice.", file=sys.stderr)
            continue
        break

    print(f"ElevenLabs request failed with status {last_status}: {last_error}", file=sys.stderr)
    return 1


def gemini_api_key() -> str:
    if present("GEMINI_API_KEY"):
        return os.environ["GEMINI_API_KEY"]
    if present("GOOGLE_API_KEY"):
        return os.environ["GOOGLE_API_KEY"]
    return ""


def wav_bytes(
    pcm: bytes,
    *,
    channels: int = 1,
    rate: int = DEFAULT_GEMINI_SAMPLE_RATE,
    sample_width: int = DEFAULT_GEMINI_SAMPLE_WIDTH,
) -> bytes:
    output = io.BytesIO()
    with wave.open(output, "wb") as wf:
        wf.setnchannels(channels)
        wf.setsampwidth(sample_width)
        wf.setframerate(rate)
        wf.writeframes(pcm)
    return output.getvalue()


def gemini_wav_options(mime_type: str) -> dict[str, int]:
    options = {
        "channels": 1,
        "rate": env_int("GEMINI_TTS_SAMPLE_RATE", DEFAULT_GEMINI_SAMPLE_RATE),
        "sample_width": DEFAULT_GEMINI_SAMPLE_WIDTH,
    }
    if not mime_type:
        return options

    media_type, *params = [part.strip() for part in mime_type.split(";")]
    if "/" in media_type:
        _, subtype = media_type.split("/", 1)
        if subtype.upper().startswith("L"):
            try:
                bits = int(subtype[1:])
                if bits > 0 and bits % 8 == 0:
                    options["sample_width"] = bits // 8
            except ValueError:
                pass

    for param in params:
        if "=" not in param:
            continue
        key, value = [part.strip() for part in param.split("=", 1)]
        if key.lower() == "rate":
            try:
                options["rate"] = int(value)
            except ValueError:
                pass
    return options


def gemini_audio_output(audio: bytes, mime_type: str) -> tuple[bytes, str, dict[str, int], bool]:
    media_type = mime_type.split(";", 1)[0].strip().lower()
    wav_options = gemini_wav_options(mime_type)
    if media_type in {"audio/wav", "audio/x-wav", "audio/wave"}:
        return audio, "wav", wav_options, False
    return wav_bytes(audio, **wav_options), "wav", wav_options, True


def gemini_inline_audio(data: dict[str, Any]) -> tuple[bytes, str]:
    for candidate in data.get("candidates", []):
        if not isinstance(candidate, dict):
            continue
        content = candidate.get("content") or {}
        if not isinstance(content, dict):
            continue
        for part in content.get("parts", []):
            if not isinstance(part, dict):
                continue
            inline = part.get("inlineData") or part.get("inline_data")
            if isinstance(inline, dict) and inline.get("data"):
                return base64.b64decode(str(inline["data"])), str(inline.get("mimeType") or inline.get("mime_type") or "audio/pcm")
    raise RuntimeError("Gemini response did not include inline audio data")


def gemini_prompt_prefix() -> str:
    prefix_file = env_value("GEMINI_TTS_PROMPT_PREFIX_FILE", "")
    if prefix_file:
        path = Path(prefix_file)
        if not path.is_absolute():
            path = ROOT / path
        return path.read_text(errors="ignore").strip()
    configured = env_value("GEMINI_TTS_PROMPT_PREFIX", "").strip()
    if configured:
        return configured
    if env_value("GEMINI_TTS_PROMPT_STYLE", DEFAULT_GEMINI_PROMPT_STYLE) == "software-tutorial":
        return """Synthesize speech only. Do not read the instructions, headings, or labels aloud.
Use the bracketed tags as delivery instructions, not as spoken words.
Speak only the text inside the TRANSCRIPT section.

# AUDIO PROFILE
A clear, calm, technically precise software tutorial narrator using the Iapetus voice.

# SCENE
A clean software tutorial recording. The viewer is following along on screen while the narrator explains UI actions, code, commands, configuration files, and expected results.

# DIRECTOR'S NOTES
Style: Professional, clear, focused, and helpful. No hype, no influencer tone, no exaggerated excitement.
Accent: General American English.
Pace: Medium-slow. Slow down slightly for commands, filenames, config keys, API names, and UI menu paths.
Articulation: Pronounce technical terms cleanly. Keep code terms, acronyms, and filenames distinct.
Pauses: Add short pauses after major steps, before warnings, and after commands.
Tone: Calm authority. Confident, but not corporate-stiff.
Energy: Slightly engaged, not monotone.

# PRONUNCIATION / TERM NOTES
ClickHouse = read as "Click House"
ClickStack = read as "Click Stack"
Grafana = read as "Grafana"
RPC = read as "R-P-C"
YAML = read as "YAML"
JSON = read as "Jason" or "J-S-O-N" depending on your preference
.env = read as "dot env"
getHealth = read as "get health"

# TRANSCRIPT"""
    return ""


def gemini_tutorial_transcript(text: str) -> str:
    paragraphs = [line.strip() for line in re.split(r"\n\s*\n", text) if line.strip()]
    tagged: list[str] = []
    for index, paragraph in enumerate(paragraphs):
        spoken = re.sub(r"`([^`]+)`", r"\1", paragraph)
        lower = spoken.lower()
        if lower.startswith(("make sure", "do not", "never")):
            tag = "important"
        elif lower.startswith(("now ", "next ", "then ", "finally ")):
            tag = "instruction"
        elif lower.startswith(("because", "this ", "the ")):
            tag = "explanation"
        elif lower.startswith(("you should", "once ", "all three", "docker shows")):
            tag = "confirmation"
        else:
            tag = "informative" if index == 0 else "explanation"
        tagged.append(f"[{tag}] {spoken}")
        if index != len(paragraphs) - 1:
            tagged.append("[short pause]")
    return "\n".join(tagged)


def gemini_prompt_text(text: str, prompt_prefix: str) -> str:
    if not prompt_prefix:
        return text
    if "# TRANSCRIPT" in prompt_prefix.upper():
        return f"{prompt_prefix.rstrip()}\n{gemini_tutorial_transcript(text)}"
    return f"{prompt_prefix.rstrip()}\n\n{text}"


def gemini_untagged_prompt_text(text: str, prompt_prefix: str) -> str:
    spoken = re.sub(r"`([^`]+)`", r"\1", text).strip()
    if not prompt_prefix:
        return spoken
    prefix = re.sub(r"(?im)^# TRANSCRIPT\s*$", "", prompt_prefix).rstrip()
    return f"{prefix}\n\n{spoken}"


def post_gemini_tts(url: str, api_key: str, prompt_text: str, temperature: float, voice: str) -> requests.Response:
    return requests.post(
        url,
        headers={
            "x-goog-api-key": api_key,
            "content-type": "application/json",
        },
        json={
            "contents": [{"role": "user", "parts": [{"text": prompt_text}]}],
            "generationConfig": {
                "temperature": temperature,
                "responseModalities": ["AUDIO"],
                "speechConfig": {
                    "voiceConfig": {
                        "prebuiltVoiceConfig": {
                            "voiceName": voice,
                        }
                    }
                },
            },
        },
        timeout=180,
    )


def generate_gemini_audio(name: str, text: str) -> int:
    api_key = gemini_api_key()
    if not api_key:
        print("Missing Gemini env: GEMINI_API_KEY or GOOGLE_API_KEY", file=sys.stderr)
        return 1

    model = os.environ.get("GEMINI_TTS_MODEL", DEFAULT_GEMINI_MODEL)
    voice = os.environ.get("GEMINI_TTS_VOICE", DEFAULT_GEMINI_VOICE)
    temperature = float(env_value("GEMINI_TTS_TEMPERATURE", "1"))
    try:
        prompt_prefix = gemini_prompt_prefix()
    except Exception as exc:
        print(f"Gemini prompt prefix failed: {exc}", file=sys.stderr)
        return 1
    prompt_text = gemini_prompt_text(text, prompt_prefix)
    url = GEMINI_TTS_URL_TEMPLATE.format(model=model)
    response = post_gemini_tts(url, api_key, prompt_text, temperature, voice)
    prompt_mode = "tagged-transcript" if "# TRANSCRIPT" in prompt_prefix.upper() else "plain"
    if response.status_code == 400 and prompt_mode == "tagged-transcript":
        fallback_text = gemini_untagged_prompt_text(text, prompt_prefix)
        fallback = post_gemini_tts(url, api_key, fallback_text, temperature, voice)
        if fallback.status_code < 400:
            print(f"Gemini tagged prompt rejected for {name}; retried with untagged transcript.", file=sys.stderr)
            response = fallback
            prompt_mode = "untagged-transcript-fallback"
    if response.status_code >= 400:
        print(f"Gemini TTS request failed with status {response.status_code}: {request_error(response)}", file=sys.stderr)
        return 1

    try:
        pcm, mime_type = gemini_inline_audio(response.json())
    except Exception as exc:
        print(f"Gemini TTS response parse failed: {exc}", file=sys.stderr)
        return 1

    audio_output, extension, wav_options, wrapped_raw_audio = gemini_audio_output(pcm, mime_type)
    write_audio_file(
        name,
        audio_output,
        extension,
        {
            "name": name,
            "provider": "gemini",
            "model_id": model,
            "voice_name": voice,
            "mime_type": mime_type,
            "output_format": "wav",
            "sample_rate": wav_options["rate"],
            "sample_width": wav_options["sample_width"],
            "temperature": temperature,
            "prompt_prefix": bool(prompt_prefix),
            "prompt_mode": prompt_mode,
            "wrapped_raw_audio": wrapped_raw_audio,
        },
    )
    return 0


def alibaba_text_chunks(text: str) -> list[str]:
    limit = max(1, env_int("ALIBABA_DASHSCOPE_MAX_CHARS", DEFAULT_ALIBABA_DASHSCOPE_MAX_CHARS))
    normalized = " ".join(line.strip() for line in text.splitlines() if line.strip())
    if len(normalized) <= limit:
        return [normalized]

    chunks: list[str] = []
    current = ""
    pieces = re.split(r"(?<=[.!?])\s+", normalized)
    for piece in pieces:
        piece = piece.strip()
        if not piece:
            continue
        if len(piece) > limit:
            words = piece.split()
            for word in words:
                candidate = f"{current} {word}".strip()
                if len(candidate) <= limit:
                    current = candidate
                else:
                    if current:
                        chunks.append(current)
                    current = word
            continue
        candidate = f"{current} {piece}".strip()
        if len(candidate) <= limit:
            current = candidate
        else:
            if current:
                chunks.append(current)
            current = piece
    if current:
        chunks.append(current)
    return chunks


def concat_audio_chunks(name: str, chunks: list[bytes], extension: str, metadata: dict[str, Any]) -> Path:
    if len(chunks) == 1:
        return write_audio_file(name, chunks[0], extension, metadata)

    if not shutil.which("ffmpeg"):
        raise RuntimeError("ffmpeg is required to concatenate Alibaba Cloud TTS chunks")

    output = audio_output_path(name, extension)
    tmp_dir = audio_dir() / f".{name}-chunks"
    if tmp_dir.exists():
        shutil.rmtree(tmp_dir)
    tmp_dir.mkdir(parents=True)
    try:
        list_path = tmp_dir / "concat.txt"
        lines = []
        for index, chunk in enumerate(chunks):
            chunk_path = tmp_dir / f"chunk-{index:03d}.{extension}"
            chunk_path.write_bytes(chunk)
            lines.append(f"file '{chunk_path}'")
        list_path.write_text("\n".join(lines) + "\n")
        if extension == "wav":
            audio_codec = ["-c:a", "pcm_s16le"]
        elif extension == "mp3":
            audio_codec = ["-c:a", "libmp3lame"]
        else:
            audio_codec = ["-c", "copy"]
        result = subprocess.run(
            [
                "ffmpeg",
                "-hide_banner",
                "-loglevel",
                "error",
                "-y",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                str(list_path),
                *audio_codec,
                str(output),
            ],
            capture_output=True,
            text=True,
        )
        if result.returncode != 0:
            raise RuntimeError(result.stderr.strip() or "ffmpeg failed to concatenate Alibaba Cloud TTS chunks")
    finally:
        shutil.rmtree(tmp_dir, ignore_errors=True)

    write_audio_metadata(output, metadata)
    print(f"audio_written: {output.relative_to(ROOT)}")
    return output


def dashscope_sdk_available() -> bool:
    try:
        import dashscope  # noqa: F401
        from dashscope.audio.tts_v2 import SpeechSynthesizer  # noqa: F401
    except Exception:
        return False
    return True


def alibaba_dashscope_api_key() -> str:
    if present("DASHSCOPE_API_KEY"):
        return os.environ["DASHSCOPE_API_KEY"]
    if present("ALIBABA_DASHSCOPE_API_KEY"):
        return os.environ["ALIBABA_DASHSCOPE_API_KEY"]
    return ""


def alibaba_dashscope_websocket_url() -> str:
    return env_value(
        "DASHSCOPE_BASE_WEBSOCKET_API_URL",
        env_value("ALIBABA_DASHSCOPE_BASE_WEBSOCKET_API_URL", DEFAULT_DASHSCOPE_BASE_WEBSOCKET_API_URL),
    )


def alibaba_dashscope_audio_format() -> Any | None:
    format_name = env_value("ALIBABA_DASHSCOPE_AUDIO_FORMAT", "")
    if not format_name:
        return None
    from dashscope.audio.tts_v2 import AudioFormat

    attr = format_name.strip().upper()
    if hasattr(AudioFormat, attr):
        return getattr(AudioFormat, attr)
    raise RuntimeError(f"Unsupported DashScope audio format: {format_name}")


def alibaba_dashscope_extension() -> str:
    format_name = env_value("ALIBABA_DASHSCOPE_AUDIO_FORMAT", "").upper()
    if format_name.startswith("WAV_"):
        return "wav"
    if format_name and not format_name.startswith("MP3_"):
        raise RuntimeError("Only DashScope MP3 or WAV output formats are supported by demo assembly")
    return DEFAULT_ALIBABA_DASHSCOPE_FORMAT


def alibaba_dashscope_synthesizer_kwargs() -> dict[str, Any]:
    kwargs: dict[str, Any] = {
        "model": env_value("ALIBABA_DASHSCOPE_MODEL", DEFAULT_ALIBABA_DASHSCOPE_MODEL),
        "voice": env_value("ALIBABA_DASHSCOPE_VOICE", DEFAULT_ALIBABA_DASHSCOPE_VOICE),
    }
    audio_format = alibaba_dashscope_audio_format()
    if audio_format is not None:
        kwargs["format"] = audio_format
    if present("ALIBABA_DASHSCOPE_VOLUME"):
        kwargs["volume"] = env_int("ALIBABA_DASHSCOPE_VOLUME", 50)
    for env_key, request_key in (
        ("ALIBABA_DASHSCOPE_SPEECH_RATE", "speech_rate"),
        ("ALIBABA_DASHSCOPE_PITCH_RATE", "pitch_rate"),
    ):
        if present(env_key):
            kwargs[request_key] = float(os.environ[env_key])
    additional_params: dict[str, Any] = {}
    if present("ALIBABA_DASHSCOPE_BIT_RATE"):
        additional_params["bit_rate"] = env_int("ALIBABA_DASHSCOPE_BIT_RATE", 32)
    if additional_params:
        kwargs["additional_params"] = additional_params
    return kwargs


def request_alibaba_dashscope_chunk(text: str) -> bytes:
    import dashscope
    from dashscope.audio.tts_v2 import SpeechSynthesizer

    dashscope.api_key = alibaba_dashscope_api_key()
    dashscope.base_websocket_api_url = alibaba_dashscope_websocket_url()
    synthesizer = SpeechSynthesizer(**alibaba_dashscope_synthesizer_kwargs())
    audio = synthesizer.call(text)
    if not audio:
        request_id = getattr(synthesizer, "get_last_request_id", lambda: "")()
        raise RuntimeError(f"Alibaba DashScope TTS returned no audio. request_id={request_id}")
    return audio


def generate_alibaba_audio(name: str, text: str) -> int:
    if not alibaba_dashscope_api_key():
        print("Missing Alibaba DashScope env: DASHSCOPE_API_KEY or ALIBABA_DASHSCOPE_API_KEY", file=sys.stderr)
        return 1
    if not dashscope_sdk_available():
        print("Missing Python package: dashscope. Run scripts/demo/install-recording-tools.sh.", file=sys.stderr)
        return 1

    extension = alibaba_dashscope_extension()
    chunks = alibaba_text_chunks(text)
    try:
        audio_chunks = [request_alibaba_dashscope_chunk(chunk) for chunk in chunks]
        concat_audio_chunks(
            name,
            audio_chunks,
            extension,
            {
                "name": name,
                "provider": "alibaba",
                "api": "dashscope",
                "model_id": env_value("ALIBABA_DASHSCOPE_MODEL", DEFAULT_ALIBABA_DASHSCOPE_MODEL),
                "voice_name": env_value("ALIBABA_DASHSCOPE_VOICE", DEFAULT_ALIBABA_DASHSCOPE_VOICE),
                "output_format": env_value("ALIBABA_DASHSCOPE_AUDIO_FORMAT", "default MP3"),
                "base_websocket_api_url": alibaba_dashscope_websocket_url(),
                "chunk_count": len(chunks),
                "max_chars": env_int("ALIBABA_DASHSCOPE_MAX_CHARS", DEFAULT_ALIBABA_DASHSCOPE_MAX_CHARS),
            },
        )
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        return 1
    return 0


def generate_audio(name: str, text: str) -> int:
    provider = tts_provider()
    if provider == "elevenlabs":
        return generate_elevenlabs_audio(name, text)
    if provider == "gemini":
        return generate_gemini_audio(name, text)
    if provider == "alibaba":
        return generate_alibaba_audio(name, text)
    print(f"Unsupported TTS_PROVIDER: {provider}", file=sys.stderr)
    return 1


def generate_scene(scene: str) -> int:
    return generate_audio(scene, scene_text(scene))


def smoke() -> int:
    return generate_audio("tts-smoke", "ClickHouse sink tutorial voice test.")


def scene_ids() -> list[str]:
    runbook = ROOT / "scripts/demo/runbook.yaml"
    try:
        import yaml
    except Exception as exc:
        raise RuntimeError("pyyaml is required") from exc
    data = yaml.safe_load(runbook.read_text())
    return [scene["id"] for scene in data["scenes"]]


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("status")
    sub.add_parser("check")
    sub.add_parser("smoke")
    voices = sub.add_parser("voices")
    voices.add_argument("--all", action="store_true", help="List all returned voices instead of default voices only.")
    select = sub.add_parser("select-free")
    select.add_argument("--write-env", action="store_true", help="Write selected voice ID to .env.demo.local without printing it.")
    scene = sub.add_parser("scene")
    scene.add_argument("scene")
    sub.add_parser("all")
    args = parser.parse_args()
    if args.cmd == "status":
        return status()
    if args.cmd == "check":
        return check()
    if args.cmd == "smoke":
        return smoke()
    if args.cmd == "voices":
        return list_voices(args.all)
    if args.cmd == "select-free":
        return select_free(args.write_env)
    if args.cmd == "scene":
        return generate_scene(args.scene)
    code = 0
    for scene_id in scene_ids():
        code = generate_scene(scene_id) or code
    return code


if __name__ == "__main__":
    raise SystemExit(main())
