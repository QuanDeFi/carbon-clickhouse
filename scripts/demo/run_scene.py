#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import time
from pathlib import Path

import yaml


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


def runbook() -> dict:
    return yaml.safe_load((ROOT / "scripts/demo/runbook.yaml").read_text())


def scene_map() -> dict[str, dict]:
    return {scene["id"]: scene for scene in runbook()["scenes"]}


def ensure_display() -> None:
    env = os.environ.copy()
    display = os.environ.get("DEMO_DISPLAY", ":95")
    expected_size = os.environ.get("DEMO_SCREEN_SIZE", "1920x1080")
    probe = subprocess.run(["xdpyinfo", "-display", display], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
    if probe.returncode == 0 and f"dimensions:    {expected_size} pixels" in probe.stdout:
        os.environ.setdefault("DISPLAY", display)
        return
    subprocess.run([str(ROOT / "scripts/demo/start-display.sh")], cwd=ROOT, env=env, check=True)
    os.environ.setdefault("DISPLAY", display)


def run(cmd: list[str], *, check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=ROOT, env=os.environ.copy(), check=check)


def maybe_voice(scene_id: str, no_voice: bool, dry_run: bool) -> bool:
    audio = ROOT / os.environ.get("DEMO_AUDIO_DIR", "demo-artifacts/audio") / f"{scene_id}.mp3"
    if audio.is_file():
        return True
    if no_voice:
        return False
    if dry_run:
        print(f"would generate voice: {scene_id}")
        return False
    run([sys.executable, str(ROOT / "scripts/demo/voice.py"), "scene", scene_id])
    return audio.is_file()


def run_scene(scene: dict, args: argparse.Namespace) -> int:
    scene_id = scene["id"]
    scene_type = scene["type"]
    started = time.time()
    report = {
        "scene": scene_id,
        "type": scene_type,
        "started_at": started,
        "steps": [],
    }

    try:
        has_audio = maybe_voice(scene_id, args.no_voice, args.dry_run)
        if scene_type in {"terminal", "browser"}:
            if args.dry_run:
                print(f"would ensure display for {scene_id}")
            else:
                ensure_display()

        recording = False
        if scene_type != "voice_only" and not args.no_record:
            if args.dry_run:
                print(f"would start recording: {scene_id}")
            else:
                run([sys.executable, str(ROOT / "scripts/demo/record.py"), "start", "--scene", scene_id])
                recording = True

        try:
            if scene_type == "terminal":
                command = scene["command"].split()
                terminal_cmd = [str(ROOT / "scripts/demo/terminal-scene.sh"), scene_id, *command]
                report["steps"].append({"terminal": command})
                if args.dry_run:
                    print("would run: " + " ".join(terminal_cmd))
                else:
                    run(terminal_cmd)
            elif scene_type == "browser":
                specs = scene["playwright"]
                if isinstance(specs, str):
                    specs = [specs]
                for spec_path in specs:
                    spec = ROOT / spec_path
                    command = [
                        "npm",
                        "--prefix",
                        str(ROOT / "scripts/demo/playwright"),
                        "exec",
                        "playwright",
                        "test",
                        spec.name,
                        "--headed",
                        "--config",
                        "playwright.config.ts",
                    ]
                    report["steps"].append({"playwright": command})
                    if args.dry_run:
                        print("would run: " + " ".join(command))
                    else:
                        subprocess.run(command, cwd=ROOT / "scripts/demo/playwright", env=os.environ.copy(), check=True)
            else:
                report["steps"].append({"voice_only": True})
        finally:
            if recording:
                run([sys.executable, str(ROOT / "scripts/demo/record.py"), "stop", "--scene", scene_id], check=False)

        if has_audio and not args.no_assemble:
            if args.dry_run:
                print(f"would assemble: {scene_id}")
            else:
                run([sys.executable, str(ROOT / "scripts/demo/assemble.py"), "scene", scene_id])
        report["status"] = "ok"
        return 0
    except subprocess.CalledProcessError as exc:
        report["status"] = "failed"
        report["exit_code"] = exc.returncode
        return exc.returncode
    finally:
        reports = ROOT / "demo-artifacts/reports"
        reports.mkdir(parents=True, exist_ok=True)
        report["finished_at"] = time.time()
        (reports / f"{scene_id}.json").write_text(json.dumps(report, indent=2))


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser()
    parser.add_argument("scene", nargs="?")
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--no-voice", action="store_true")
    parser.add_argument("--no-record", action="store_true")
    parser.add_argument("--no-assemble", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--human", action="store_true")
    args = parser.parse_args()
    if args.human:
        if not args.all:
            print("--human currently runs the full human rehearsal; use --all", file=sys.stderr)
            return 2
        command = [sys.executable, str(ROOT / "scripts/demo/run_human_rehearsal.py")]
        if args.no_voice:
            command.append("--no-voice")
        return subprocess.run(command, cwd=ROOT, env=os.environ.copy()).returncode
    scenes = scene_map()
    if args.all:
        code = 0
        for scene in scenes.values():
            code = run_scene(scene, args) or code
        return code
    if not args.scene or args.scene not in scenes:
        print("Provide a valid scene id or --all", file=sys.stderr)
        return 2
    return run_scene(scenes[args.scene], args)


if __name__ == "__main__":
    raise SystemExit(main())
