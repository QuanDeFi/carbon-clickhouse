#!/usr/bin/env python3
from __future__ import annotations

import argparse
import contextlib
import io
import json
import os
import signal
import shutil
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
LOCAL_CLICKHOUSE_URL = "http://carbon:carbon@localhost:8123"


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


def artifact_dir(name: str) -> Path:
    path = ROOT / os.environ.get(name, "demo-artifacts/recordings")
    path.mkdir(parents=True, exist_ok=True)
    return path


def backend() -> str:
    return os.environ.get("RECORDER_BACKEND", "ffmpeg_x11")


def scene_paths(scene: str) -> tuple[Path, Path]:
    recordings = artifact_dir("DEMO_RECORDING_DIR")
    return recordings / f"{scene}.mp4", recordings / f"{scene}.recording.json"


def status() -> int:
    print(f"recorder_backend: {backend()}")
    print(f"display: {os.environ.get('DEMO_DISPLAY', ':95')}")
    print(f"screen_size: {os.environ.get('DEMO_SCREEN_SIZE', '1920x1080')}")
    print(f"fps: {os.environ.get('DEMO_FPS', '30')}")
    ffmpeg = subprocess.run(["bash", "-lc", "command -v ffmpeg"], capture_output=True, text=True)
    print(f"ffmpeg: {'present' if ffmpeg.returncode == 0 else 'missing'}")
    obs = subprocess.run(["bash", "-lc", "command -v obs"], capture_output=True, text=True)
    print(f"OBS installed: {'yes' if obs.returncode == 0 else 'no'}")
    obs_process = subprocess.run(["bash", "-lc", "pgrep -x obs >/dev/null"], capture_output=True, text=True)
    print(f"OBS process running: {'yes' if obs_process.returncode == 0 else 'no'}")
    port = os.environ.get("OBS_WEBSOCKET_PORT", "4455")
    port_check = subprocess.run(
        ["bash", "-lc", f"ss -ltn 2>/dev/null | awk '$4 ~ /:{port}$/ {{found=1}} END {{exit found ? 0 : 1}}'"],
        capture_output=True,
        text=True,
    )
    print(f"OBS websocket port listening: {'yes' if port_check.returncode == 0 else 'no'}")
    print(f"OBS websocket auth: {'configured' if os.environ.get('OBS_WEBSOCKET_PASSWORD') else 'unknown'}")
    if backend() == "obs":
        return obs_status()
    print("Connection test: skipped for ffmpeg_x11 backend")
    return 0


def start_ffmpeg(scene: str) -> int:
    output, meta = scene_paths(scene)
    display = os.environ.get("DEMO_DISPLAY", ":95")
    size = os.environ.get("DEMO_SCREEN_SIZE", "1920x1080")
    fps = os.environ.get("DEMO_FPS", "30")
    log_path = output.with_suffix(".ffmpeg.log")
    cmd = [
        "ffmpeg",
        "-y",
        "-video_size",
        size,
        "-framerate",
        fps,
        "-f",
        "x11grab",
        "-draw_mouse",
        "0",
        "-i",
        display,
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-pix_fmt",
        "yuv420p",
        str(output),
    ]
    log = log_path.open("wb")
    proc = subprocess.Popen(cmd, stdout=log, stderr=subprocess.STDOUT, cwd=ROOT)
    meta.write_text(
        json.dumps(
            {
                "scene": scene,
                "backend": "ffmpeg_x11",
                "pid": proc.pid,
                "output": str(output.relative_to(ROOT)),
                "log": str(log_path.relative_to(ROOT)),
                "started_at": time.time(),
            },
            indent=2,
        )
    )
    print(f"recording_started: {scene}")
    print(f"pid: {proc.pid}")
    print(f"output: {output.relative_to(ROOT)}")
    return 0


def stop_ffmpeg(scene: str) -> int:
    output, meta = scene_paths(scene)
    if not meta.is_file():
        print(f"recording metadata missing for {scene}", file=sys.stderr)
        return 1
    data = json.loads(meta.read_text())
    pid = int(data["pid"])
    try:
        os.kill(pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    else:
        for _ in range(50):
            try:
                os.kill(pid, 0)
            except ProcessLookupError:
                break
            time.sleep(0.1)
        else:
            os.kill(pid, signal.SIGKILL)
    data["stopped_at"] = time.time()
    meta.write_text(json.dumps(data, indent=2))
    print(f"recording_stopped: {scene}")
    print(f"output: {output.relative_to(ROOT)}")
    return 0


def obs_client():
    try:
        import obsws_python as obs
    except Exception as exc:  # pragma: no cover - dependency checked at runtime.
        raise RuntimeError("obsws-python is not installed") from exc
    with contextlib.redirect_stderr(io.StringIO()):
        return obs.ReqClient(
            host=os.environ.get("OBS_WEBSOCKET_HOST", "127.0.0.1"),
            port=int(os.environ.get("OBS_WEBSOCKET_PORT", "4455")),
            password=os.environ.get("OBS_WEBSOCKET_PASSWORD", ""),
            timeout=3,
        )


def obs_status() -> int:
    try:
        client = obs_client()
        version = client.get_version()
        state = client.get_record_status()
    except Exception as exc:
        print("obs_websocket: unavailable")
        print("Connection test: failed")
        print(str(exc))
        print("recommendation: use RECORDER_BACKEND=ffmpeg_x11")
        return 1
    print(f"obs_websocket: ok")
    print("Connection test: ok")
    print(f"obs_version: {getattr(version, 'obs_version', 'unknown')}")
    print(f"recording: {getattr(state, 'output_active', 'unknown')}")
    return 0


def start_obs(scene: str) -> int:
    try:
        client = obs_client()
        state = client.get_record_status()
        if not getattr(state, "output_active", False):
            client.start_record()
    except Exception as exc:
        print("OBS recording start failed; use RECORDER_BACKEND=ffmpeg_x11", file=sys.stderr)
        print(str(exc), file=sys.stderr)
        return 1
    _, meta = scene_paths(scene)
    meta.write_text(json.dumps({"scene": scene, "backend": "obs", "started_at": time.time()}, indent=2))
    print(f"obs_recording_started: {scene}")
    return 0


def stop_obs(scene: str) -> int:
    _, meta = scene_paths(scene)
    output_path = None
    try:
        client = obs_client()
        state = client.get_record_status()
        if getattr(state, "output_active", False):
            result = client.stop_record()
            output_path = getattr(result, "output_path", None)
    except Exception as exc:
        print("OBS recording stop failed", file=sys.stderr)
        print(str(exc), file=sys.stderr)
        return 1
    copied_to = None
    if output_path:
        source = Path(output_path)
        for _ in range(50):
            if source.is_file() and source.stat().st_size > 0:
                break
            time.sleep(0.2)
        if source.is_file() and source.stat().st_size > 0:
            target = artifact_dir("DEMO_RECORDING_DIR") / f"{scene}{source.suffix or '.mkv'}"
            shutil.move(str(source), target)
            copied_to = str(target.relative_to(ROOT))
    data = {"scene": scene, "backend": "obs", "stopped_at": time.time()}
    if output_path:
        data["obs_output_path"] = output_path
    if copied_to:
        data["output"] = copied_to
    meta.write_text(json.dumps(data, indent=2))
    print(f"obs_recording_stopped: {scene}")
    if copied_to:
        print(f"output: {copied_to}")
    return 0


def main() -> int:
    load_env()
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="cmd", required=True)
    sub.add_parser("status")
    start = sub.add_parser("start")
    start.add_argument("--scene", required=True)
    stop = sub.add_parser("stop")
    stop.add_argument("--scene", required=True)
    args = parser.parse_args()

    if args.cmd == "status":
        return status()
    if backend() == "obs":
        return start_obs(args.scene) if args.cmd == "start" else stop_obs(args.scene)
    return start_ffmpeg(args.scene) if args.cmd == "start" else stop_ffmpeg(args.scene)


if __name__ == "__main__":
    raise SystemExit(main())
