# Carbon ClickHouse Tutorial Recording Automation

This folder contains deterministic automation for recording the Carbon
ClickHouse sink tutorial. The source tutorial path is:

```sh
crates/core/src/clickhouse/docs/ClickHouse Sink Tutorial Curriculum.md
```

The runbook for recording is:

```sh
scripts/demo/runbook.yaml
```

## Architecture

- Headless display: Xvfb on `${DEMO_DISPLAY:-:95}`.
- Optional observation UI: x11vnc on `${DEMO_VNC_PORT:-5903}` and noVNC/websockify on `${DEMO_NOVNC_PORT:-6083}`.
- Terminal scenes: wide dark-theme `xterm` windows with smaller readable text and visibly typed commands.
- Human-style browser scenes: Chromium app windows launched visibly on Xvfb, with browser chrome hidden and dark rendering where possible.
- Default recorder: FFmpeg `x11grab`.
- Optional recorder: OBS through obs-websocket.
- Voiceover: ElevenLabs.
- Assembly: FFmpeg.

FFmpeg x11grab is the default because this RPC node is headless and OBS is not
required for the current environment. OBS support is implemented as an optional
backend for machines where OBS and obs-websocket are available.

## Initialize Environment

Create `.env.demo.local` from the existing example env files:

```sh
scripts/demo/init-env-from-examples.sh
```

Then edit `.env.demo.local` and fill any missing values:

```sh
$EDITOR .env.demo.local
```

Do not commit `.env.demo.local`.

## Install Tools

```sh
scripts/demo/install-recording-tools.sh
```

This installs or verifies system FFmpeg, optional OBS, X11 terminal tooling,
the Python demo venv, and demo-local Playwright dependencies.

## Start Display

```sh
scripts/demo/start-display.sh
```

The command prints:

```text
DISPLAY=:95
noVNC: http://localhost:6083/vnc.html
```

noVNC is for observation only. Recording does not depend on it.
The selected display and ports are also written to `demo-artifacts/display.env`.
If the selected display or ports are already used by another local VNC session,
keep that session untouched and use explicit overrides or `--auto`:

```sh
DEMO_DISPLAY=:96 DEMO_VNC_PORT=5904 DEMO_NOVNC_PORT=6084 scripts/demo/start-display.sh
scripts/demo/start-display.sh --auto
```

## Preflight

```sh
scripts/demo/preflight.sh --no-elevenlabs
```

Use `--no-obs` when using the default FFmpeg backend:

```sh
scripts/demo/preflight.sh --no-elevenlabs --no-obs
```

## Generate Voice

Free ElevenLabs accounts cannot use every library voice. The demo helper can
query voices available to the account and select a default/free-usable voice
without printing the voice ID:

```sh
source .venv-demo/bin/activate
python scripts/demo/voice.py status
python scripts/demo/voice.py voices
python scripts/demo/voice.py select-free --write-env
python scripts/demo/voice.py smoke
```

Generate all scene voiceovers after the smoke test passes:

```sh
source .venv-demo/bin/activate
python scripts/demo/voice.py all
```

## Run One Scene

```sh
source .venv-demo/bin/activate
python scripts/demo/run_scene.py scene-02-local-setup
```

For a no-voice rehearsal:

```sh
python scripts/demo/run_scene.py scene-02-local-setup --no-voice
```

## Run All Scenes

```sh
source .venv-demo/bin/activate
python scripts/demo/run_scene.py --all
```

## Human-Style No-Audio Review

Use this before final voiceover. It records a directed screen performance:
visible terminal typing, readable pauses, dark browser slides, ClickHouse `/play`
inspection, Grafana dashboards after both examples, ClickStack query-log
inspection, video validation, screenshots, contact sheet, and current review
files.

The current story is live-first: verify local services, start Jupiter live in
one terminal, start Token Program live in a second terminal, inspect Grafana and
ClickStack while both examples are running, inspect landing table data in
ClickHouse `/play`, demonstrate async-wait inserts, and stop the live terminals
during the final production-boundaries slide.

Terminal scenes reuse named shells where appropriate, pause briefly before
pressing Enter, and leave read time at scene boundaries so the output can be
consumed. Typing uses a seeded human-style profile: skewed per-character delays,
short burst pauses, slower shell-symbol timing, boundary pauses, and randomized
pre-Enter pauses. The terminal driver also waits for the shell prompt to return
before typing the next command, so long-running examples cannot be interrupted
by the following command being typed into their active output.
The terminal frame defaults to equal margins on all four screen edges and a
larger readable dark-theme font. ClickHouse `/play` SQL is visibly typed into
the editor, and sample queries should include enough columns to explain the
landing-row data rather than only showing minimal counts.
The current visual target is roughly three to three and a half minutes. If a run
drifts toward four minutes or more, shorten scene content rather than returning
to fixed robotic typing.
The human rehearsal defaults to 60fps so the Mission-Control-style window
transition keeps the smoother transform-based motion from the focused probe.
Before cleaning screenshot folders, the runner preserves prior good browser
screenshots into `demo-artifacts/window-transition-seeds/`; missing seeds are
safe and fall back to the transition renderer's muted placeholder tiles.

```sh
source .venv-demo/bin/activate
export RECORDER_BACKEND=ffmpeg_x11
export DEMO_DISPLAY=:95 DISPLAY=:95
export DEMO_SCREEN_SIZE=1920x1080
export DEMO_VNC_PORT=5903 DEMO_NOVNC_PORT=6083 DEMO_FPS=60
DEMO_ALLOW_CLICKHOUSE_RESET=true python scripts/demo/run_human_rehearsal.py --no-voice
```

Outputs land under `demo-artifacts/review-human/`.

The recorder disables mouse capture, so the cursor should not appear in the
review videos.

The rehearsal also generates subtitles from the scene narration markdown:

- Sidecar SRT: `demo-artifacts/review-human/subtitles/clickhouse-sink-tutorial-human-no-audio.srt`
- Sidecar WebVTT: `demo-artifacts/review-human/subtitles/clickhouse-sink-tutorial-human-no-audio.vtt`
- Burned-in review video: `demo-artifacts/review-human/videos/clickhouse-sink-tutorial-human-no-audio-subtitled.mp4`

To regenerate subtitles for the current review video without re-recording:

```sh
source .venv-demo/bin/activate
python scripts/demo/subtitles.py
```

Current browser-scene flow:

- `scene-04`: ClickHouse `/play` shows Jupiter counts/sample rows, then Grafana
  shows Carbon pipeline and ClickHouse sink metrics after Jupiter ingestion.
- `scene-07`: ClickHouse `/play` shows Token Program landing/sample rows,
  Grafana shows the dashboard again after Token Program ingestion, then
  ClickStack shows ClickHouse-side `system.query_log` activity.

Current theme behavior:

- Intro/outro slides are dark HTML slides.
- ClickHouse `/play` and ClickStack are darkened through deterministic
  recording-time CSS injection.
- Grafana uses its native dark theme/dashboard URL.
- Browser chrome and address bars are hidden/cropped from browser scenes.

## Stop Display

```sh
scripts/demo/stop-display.sh
```

Only PIDs created by `start-display.sh` are stopped.

## Optional OBS Backend

Set these in `.env.demo.local` if you want OBS instead of the default FFmpeg
recorder:

```env
RECORDER_BACKEND=obs
OBS_WEBSOCKET_HOST=127.0.0.1
OBS_WEBSOCKET_PORT=4455
OBS_WEBSOCKET_PASSWORD=...
```

Then check:

```sh
source .venv-demo/bin/activate
scripts/demo/start-obs.sh
RECORDER_BACKEND=obs python scripts/demo/record.py status
```

If OBS websocket is unavailable, use:

```env
RECORDER_BACKEND=ffmpeg_x11
```

Stop the backup OBS services with:

```sh
scripts/demo/stop-obs.sh
```

## Safety Commands

Detect running examples:

```sh
scripts/demo/detect-running-demo-processes.sh
```

Stop demo/example processes only when explicitly allowed:

```sh
DEMO_ALLOW_STOP_PROCESSES=true scripts/demo/stop-demo-processes.sh
```

Preview ClickHouse reset:

```sh
scripts/demo/reset-clickhouse.sh
```

Perform the reset only when explicitly allowed:

```sh
DEMO_ALLOW_CLICKHOUSE_RESET=true scripts/demo/reset-clickhouse.sh
```

The reset targets only `jupiter_swap_%landing` and
`token_program_%account_landing` tables.

## Current Environment Notes

The audited machine is headless. FFmpeg x11grab is the default recorder. OBS is
optional backup.

The unmanaged `:99` display may already be occupied at `1440x1000`. For clean
tutorial recording, use:

```sh
export DEMO_DISPLAY=:95
export DEMO_SCREEN_SIZE=1920x1080
export DEMO_VNC_PORT=5903
export DEMO_NOVNC_PORT=6083
```

Before deterministic tutorial runs, reset tutorial ClickHouse tables:

```sh
DEMO_ALLOW_CLICKHOUSE_RESET=true scripts/demo/reset-clickhouse.sh
```

ElevenLabs requires an API key in `.env.demo.local`. A voice ID can be set
manually, or `voice.py` can discover a default/free-usable voice and write it
without printing the ID:

```env
ELEVENLABS_API_KEY=<elevenlabs-api-key>
ELEVENLABS_VOICE_ID=<voice-id>
ELEVENLABS_MODEL_ID=eleven_multilingual_v2
ELEVENLABS_OUTPUT_FORMAT=mp3_44100_128
ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK=false
ELEVENLABS_AUTO_WRITE_VOICE_ID=false
```

```sh
source .venv-demo/bin/activate
python scripts/demo/voice.py select-free --write-env
python scripts/demo/voice.py smoke
```

OBS backup recording is valid only after:

```sh
scripts/demo/start-obs.sh
RECORDER_BACKEND=obs python scripts/demo/record.py status
RECORDER_BACKEND=obs python scripts/demo/record.py start --scene obs-smoke
RECORDER_BACKEND=obs python scripts/demo/record.py stop --scene obs-smoke
scripts/demo/stop-obs.sh
```
