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

- Headless display: Xvfb on `${DEMO_DISPLAY:-:96}`.
- Optional observation UI: x11vnc on `${DEMO_VNC_PORT:-5904}` and noVNC/websockify on `${DEMO_NOVNC_PORT:-6084}`.
- VS Code scenes: real desktop VS Code on the recording display for code and
  configuration walkthroughs.
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
DISPLAY=:96
noVNC: http://localhost:6084/vnc.html
```

noVNC is for observation only. Recording does not depend on it.
The selected display and ports are also written to `demo-artifacts/display.env`.
If the selected display or ports are already used by another local VNC session,
keep that session untouched and use explicit overrides or `--auto`:

```sh
DEMO_DISPLAY=:97 DEMO_VNC_PORT=5905 DEMO_NOVNC_PORT=6085 scripts/demo/start-display.sh
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

## Human-Style No-Audio Review

Use this before final voiceover. It records a directed screen performance:
VS Code code/config walkthroughs, visible terminal typing, readable pauses,
ClickHouse `/play` table-browser inspection, Grafana dashboards after both
examples, ClickStack Inserts dashboard inspection, video validation,
screenshots, contact sheet, and current review files.

The current story is live-first but config-aware: verify required local
services, show the example database/RPC config in VS Code, briefly point to the
core sink configuration surface, start Jupiter in one terminal, start Token
Program in a second terminal, inspect Grafana and ClickStack while the examples
run, and inspect generated landing table data in
ClickHouse `/play`.

Terminal scenes reuse named shells where appropriate, pause briefly before
pressing Enter, and leave read time at scene boundaries so the output can be
consumed. Typing uses a seeded human-style profile: skewed per-character delays,
short burst pauses, slower shell-symbol timing, boundary pauses, and randomized
pre-Enter pauses. The terminal driver also waits for the shell prompt to return
before typing the next command, so long-running examples cannot be interrupted
by the following command being typed into their active output.
The terminal frame defaults to equal margins on all four screen edges and a
larger readable dark-theme font. ClickHouse `/play` expands the table browser,
scrolls through generated landing tables, and opens representative tables with
queries focused on readable landing-row data rather than minimal counts.
The current visual target is roughly two and a half minutes. If a run drifts
toward four minutes or more, shorten scene content rather than returning to
fixed robotic typing.
The human rehearsal defaults to 60fps. Active window switches use a short
blackout transition; the previous Mission-Control-style transition is archived
in `scripts/demo/window_transition_mission_control.py`.

```sh
source .venv-demo/bin/activate
export RECORDER_BACKEND=ffmpeg_x11
source /home/ops/dev/vnc/recording-96/display.env
export DEMO_SCREEN_SIZE=1920x1080
export DEMO_FPS=60
DEMO_ALLOW_CLICKHOUSE_RESET=true python scripts/demo/run_human_rehearsal.py --no-voice
```

Outputs land under `demo-artifacts/review-human/`.

The recorder disables mouse capture, so the cursor should not appear in the
review videos.

Subtitles are generated separately from the visual rehearsal:

```sh
source .venv-demo/bin/activate
python scripts/demo/subtitles.py
```

Current human-scene flow:

- `scene-01-readiness-checks`: terminal health checks prove ClickHouse,
  Prometheus, Grafana, and RPC reachability are online.
- `scene-02-example-env`: VS Code shows both example `.env.example` files and
  points to `crates/core/src/clickhouse/config.rs` as the full sink config
  reference.
- `scene-03-jupiter-live`: terminal starts the Jupiter live example with
  async-wait inserts.
- `scene-04-token-live`: terminal starts the Token Program example with
  async-wait inserts.
- `scene-05-observability`: filtered Jupiter and Token Program Grafana
  dashboards show the running Carbon pipelines, then ClickStack shows
  ClickHouse-side insert rows and bytes per table.
- `scene-06-clickhouse-play`: ClickHouse `/play` uses the table browser, a
  generated table-family summary, and the largest populated landing table to
  verify generated tables, data volume, and queryable rows.

Current theme behavior:

- VS Code uses a dedicated recording profile under `/home/ops/dev/vnc`.
- ClickHouse `/play` and ClickStack are darkened through deterministic
  recording-time CSS injection without decorative backgrounds.
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
source /home/ops/dev/vnc/recording-96/display.env
export DEMO_SCREEN_SIZE=1920x1080
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
