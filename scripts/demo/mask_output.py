#!/usr/bin/env python3
from __future__ import annotations

import os
import re
import sys
from pathlib import Path


LOCAL_CLICKHOUSE_URL = "http://localhost:8123"
SECRET_KEYS = {
    "SOLANA_RPC_URL",
    "RPC_URL",
    "ELEVENLABS_API_KEY",
    "ELEVENLABS_VOICE_ID",
    "GEMINI_API_KEY",
    "GOOGLE_API_KEY",
    "DASHSCOPE_API_KEY",
    "ALIBABA_DASHSCOPE_API_KEY",
    "ALIBABA_NLS_TOKEN",
    "ALIBABA_CLOUD_ACCESS_KEY_ID",
    "ALIBABA_CLOUD_ACCESS_KEY_SECRET",
    "OBS_WEBSOCKET_PASSWORD",
}


def load_env_file(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        return values
    for raw in path.read_text(errors="ignore").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        key, value = line.split("=", 1)
        value = value.strip().strip('"').strip("'")
        values[key.strip()] = value
    return values


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def secret_values() -> list[str]:
    root = repo_root()
    merged: dict[str, str] = {}
    for path in (root / ".env.demo.local", root / "scripts/demo/.env.demo.local"):
        merged.update(load_env_file(path))
    for key, value in os.environ.items():
        if key in SECRET_KEYS or key == "DATABASE_URL":
            merged[key] = value

    values: list[str] = []
    for key, value in merged.items():
        if not value or value.startswith("<"):
            continue
        if key == "DATABASE_URL" and value == LOCAL_CLICKHOUSE_URL:
            continue
        if key in SECRET_KEYS or key == "DATABASE_URL":
            values.append(value)
    return sorted(set(values), key=len, reverse=True)


def mask(text: str) -> str:
    for value in secret_values():
        text = text.replace(value, "<redacted>")
    # Defense in depth for common RPC/API-key URL shapes.
    text = re.sub(
        r"https?://[^\s'\"<>]*(api-key|apikey|token|x-token|alchemy|helius|quicknode|syndica|chainstack|drpc|getblock|shyft)[^\s'\"<>]*",
        "<redacted-url>",
        text,
        flags=re.IGNORECASE,
    )
    text = re.sub(r"https?://[^\s'\"<>/@:]+:[^\s'\"<>/@]+@", "http://<redacted>@", text)
    text = re.sub(r"sk_[A-Za-z0-9]{16,}", "<redacted-api-key>", text)
    text = re.sub(r"sk-[A-Za-z0-9._-]{16,}", "<redacted-api-key>", text)
    text = re.sub(r"AIza[A-Za-z0-9_-]{20,}", "<redacted-api-key>", text)
    text = re.sub(r"AQ\.[A-Za-z0-9_-]{20,}", "<redacted-api-key>", text)
    text = re.sub(r"LTAI[A-Za-z0-9]{12,}", "<redacted-access-key-id>", text)
    return text


def main() -> int:
    for chunk in sys.stdin:
        sys.stdout.write(mask(chunk))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
