#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import sys
import time
from pathlib import Path
from typing import Any

import requests


ROOT = Path(__file__).resolve().parents[2]
VOICES_URL = "https://api.elevenlabs.io/v2/voices"
TTS_URL_TEMPLATE = "https://api.elevenlabs.io/v1/text-to-speech/{voice_id}"
DEFAULT_MODEL_ID = "eleven_multilingual_v2"
DEFAULT_OUTPUT_FORMAT = "mp3_44100_128"


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


def status() -> int:
    print(f"ELEVENLABS_API_KEY: {'present' if present('ELEVENLABS_API_KEY') else 'missing'}")
    print(f"ELEVENLABS_VOICE_ID: {'present' if present('ELEVENLABS_VOICE_ID') else 'missing'}")
    model = os.environ.get("ELEVENLABS_MODEL_ID", DEFAULT_MODEL_ID)
    output_format = os.environ.get("ELEVENLABS_OUTPUT_FORMAT", DEFAULT_OUTPUT_FORMAT)
    print(f"ELEVENLABS_MODEL_ID: {'present' if present('ELEVENLABS_MODEL_ID') else 'defaulted to ' + model}")
    print(f"ELEVENLABS_OUTPUT_FORMAT: {'present' if present('ELEVENLABS_OUTPUT_FORMAT') else 'defaulted to ' + output_format}")
    print(
        "ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK: "
        + ("present/enabled" if env_bool("ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK") else "missing/defaulted to false")
    )
    if not present("ELEVENLABS_API_KEY") or not present("ELEVENLABS_VOICE_ID"):
        print("Add these placeholders to .env.demo.local if voice generation is needed:")
        print("ELEVENLABS_API_KEY=<elevenlabs-api-key>")
        print("ELEVENLABS_VOICE_ID=<voice-id>")
        print(f"ELEVENLABS_MODEL_ID={DEFAULT_MODEL_ID}")
        print(f"ELEVENLABS_OUTPUT_FORMAT={DEFAULT_OUTPUT_FORMAT}")
    return 0


def require_api_key() -> bool:
    if not present("ELEVENLABS_API_KEY"):
        print("Missing ElevenLabs env: ELEVENLABS_API_KEY", file=sys.stderr)
        return False
    return True


def request_error(response: requests.Response) -> str:
    try:
        body = response.json()
    except Exception:
        body = {}
    detail = body.get("detail") if isinstance(body, dict) else None
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
    if isinstance(body, dict):
        for key in ("message", "error"):
            if body.get(key):
                return str(body[key])
    return response.text[:240]


def fetch_voices(params: dict[str, Any]) -> list[dict[str, Any]]:
    response = requests.get(
        VOICES_URL,
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
    if not require_api_key():
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
    if not require_api_key():
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


def write_audio(name: str, text: str, voice: dict[str, Any], voice_source: str) -> tuple[bool, int, str]:
    voice_id = str(voice["voice_id"])
    model_id = os.environ.get("ELEVENLABS_MODEL_ID", DEFAULT_MODEL_ID)
    output_format = os.environ.get("ELEVENLABS_OUTPUT_FORMAT", DEFAULT_OUTPUT_FORMAT)
    url = TTS_URL_TEMPLATE.format(voice_id=voice_id)
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

    audio_dir = ROOT / os.environ.get("DEMO_AUDIO_DIR", "demo-artifacts/audio")
    audio_dir.mkdir(parents=True, exist_ok=True)
    output = audio_dir / f"{name}.mp3"
    metadata = audio_dir / f"{name}.json"
    output.write_bytes(response.content)
    metadata.write_text(
        json.dumps(
            {
                "name": name,
                "output": str(output.relative_to(ROOT)),
                "bytes": output.stat().st_size,
                "model_id": model_id,
                "output_format": output_format,
                "voice_source": voice_source,
                "voice_name": voice.get("name"),
                "voice_category": voice.get("category"),
                "generated_at": time.time(),
            },
            indent=2,
        )
    )
    print(f"audio_written: {output.relative_to(ROOT)}")
    return True, response.status_code, ""


def configured_voice() -> dict[str, Any] | None:
    if not present("ELEVENLABS_VOICE_ID"):
        return None
    return {
        "voice_id": os.environ["ELEVENLABS_VOICE_ID"],
        "name": "configured",
        "category": "configured",
    }


def generate_audio(name: str, text: str) -> int:
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

    fallback_voice: dict[str, Any] | None = None
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
        ok, status_code, error = write_audio(name, text, voice, source)
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


def generate_scene(scene: str) -> int:
    return generate_audio(scene, scene_text(scene))


def smoke() -> int:
    return generate_audio("elevenlabs-smoke", "ClickHouse sink tutorial voice test.")


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
