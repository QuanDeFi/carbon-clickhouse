#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


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


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser(description="Compatibility wrapper for the current human tutorial rehearsal.")
    parser.add_argument("scene", nargs="?", help="Single-scene mechanical recording has been removed.")
    parser.add_argument("--all", action="store_true", help="Run the current full human rehearsal.")
    parser.add_argument("--human", action="store_true", help="Accepted for backward compatibility.")
    parser.add_argument("--no-voice", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--no-record", action="store_true", help="Unsupported by the human rehearsal wrapper.")
    parser.add_argument("--no-assemble", action="store_true", help="Unsupported by the human rehearsal wrapper.")
    args = parser.parse_args()

    if args.scene:
        print(
            "Single-scene mechanical recording was removed. Use scripts/demo/run_human_rehearsal.py for the current tutorial flow.",
            file=sys.stderr,
        )
        return 2
    if not args.all:
        print("Use --all to run the current human tutorial rehearsal.", file=sys.stderr)
        return 2
    if args.no_record or args.no_assemble:
        print("--no-record and --no-assemble are not supported by the human rehearsal wrapper.", file=sys.stderr)
        return 2

    command = [sys.executable, str(ROOT / "scripts/demo/run_human_rehearsal.py")]
    if args.no_voice:
        command.append("--no-voice")
    if args.dry_run:
        print(" ".join(command))
        return 0
    return subprocess.run(command, cwd=ROOT, env=os.environ.copy()).returncode


if __name__ == "__main__":
    raise SystemExit(main())
