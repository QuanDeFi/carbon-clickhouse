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
- Voiceover: configurable TTS provider; ElevenLabs is the default.
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
DEMO_DISPLAY=:<display> DEMO_VNC_PORT=<vnc-port> DEMO_NOVNC_PORT=<novnc-port> scripts/demo/start-display.sh
scripts/demo/start-display.sh --auto
```

## Preflight

```sh
scripts/demo/preflight.sh --no-tts
```

Use `--no-obs` when using the default FFmpeg backend:

```sh
scripts/demo/preflight.sh --no-tts --no-obs
```

## Generate Voice

`voice.py` supports `TTS_PROVIDER=elevenlabs`, `TTS_PROVIDER=gemini`, and
`TTS_PROVIDER=alibaba`. ElevenLabs remains the default. For Gemini, set a
Google AI Studio/Gemini API key and the optional model/voice fields. For
Alibaba Cloud, set a Model Studio DashScope API key and the optional
CosyVoice model/voice fields.

Free ElevenLabs accounts cannot use every library voice. When using
`TTS_PROVIDER=elevenlabs`, the demo helper can query voices available to the
account and select a default/free-usable voice without printing the voice ID:

```sh
source .venv-demo/bin/activate
python scripts/demo/voice.py status
python scripts/demo/voice.py check
python scripts/demo/voice.py voices
python scripts/demo/voice.py select-free --write-env
python scripts/demo/voice.py smoke
```

Generate all scene voiceovers after the smoke test passes:

```sh
source .venv-demo/bin/activate
python scripts/demo/voice.py all
```

## Place Voiceover

Gemini and other TTS providers generate coherent scene-level audio. The final
tutorial sync is handled separately: `place_voiceover.py` cuts the scene audio
at detected silence/speech boundaries, places each utterance on the review-video
timeline using explicit overrides, browser workflow events, and runbook action
timings, then muxes the final voiceover video. Subtitle cues are fallback and
comparison metadata only.

```sh
source .venv-demo/bin/activate
TTS_PROVIDER=gemini python scripts/demo/voice.py all
python scripts/demo/place_voiceover.py
python scripts/demo/subtitles.py --source voiceover
python scripts/demo/build_timeline_review.py
```

Outputs:

- `demo-artifacts/review-human/audio/voiceover-timeline.wav`
- `demo-artifacts/review-human/audio/voiceover-placement-report.json`
- `demo-artifacts/review-human/videos/clickhouse-sink-tutorial-human-voiceover.mp4`
- `demo-artifacts/review-human/subtitles/clickhouse-sink-tutorial-human-voiceover.{srt,vtt,ass}`
- `demo-artifacts/review-human/videos/clickhouse-sink-tutorial-human-voiceover-subtitled.mp4`

Final subtitle timing should be generated from the placement report. The text
comes from the known Gemini prompt transcript carried through audio analysis and
placement, not from external snippet transcriptions. External transcriptions are
validation aids only because ASR can change words such as code terms,
filenames, or product names.

For browser review from another machine, serve the exported review root with
HTTP byte-range support so MP4 scrubbing works:

```sh
python scripts/demo/serve_timeline_review.py \
  --directory demo-artifacts/review-human \
  --host 0.0.0.0 \
  --port 18085
```

## Export Snippet Clips for Transcription Review

When source-audio segmentation needs human validation, export each detected
utterance snippet as a standalone WAV before calling any external transcription
service:

```sh
source .venv-demo/bin/activate
python scripts/demo/export_audio_snippets.py
python scripts/demo/build_timeline_review.py
```

Outputs:

- `demo-artifacts/review-human/audio-snippets/snippet-manifest.json`
- `demo-artifacts/review-human/audio-snippets/<scene>/<scene>--uNN.wav`

The manifest is service-agnostic and gives each clip a stable `snippet_id`.
After an external service returns transcripts, write them to:

```text
demo-artifacts/review-human/audio-snippets/snippet-transcriptions.json
```

Expected result shape:

```json
{
  "provider": "external-service-name",
  "items": [
    {
      "snippet_id": "scene-05-observability--u01",
      "text": "transcribed snippet text",
      "confidence": 0.98
    }
  ]
}
```

Then rebuild `subtitle-alignment-timeline.html`; the Scene Snippet Inspector
will show the exported clip link and the external transcript beside the current
forced-aligned text.

## Human-Style No-Audio Review

Use this before final voiceover. It records a directed screen performance:
VS Code code/config walkthroughs, visible terminal typing, readable pauses,
ClickHouse `/play` table-browser inspection, Grafana dashboards after both
examples, ClickStack Inserts dashboard inspection, video and timing validation,
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
The current visual target is roughly four and a half minutes. If a run drifts
beyond the narration-driven scene timing, shorten scene content rather than
returning to fixed robotic typing.
The human rehearsal defaults to 60fps. Active window switches use a short
blackout transition with previous/target screenshots when available; the
previous Mission-Control-style transition is archived in
`scripts/demo/window_transition_mission_control.py`.

```sh
source .venv-demo/bin/activate
export RECORDER_BACKEND=ffmpeg_x11
source "$DEMO_RECORDING_DISPLAY_ENV"
export DEMO_SCREEN_SIZE=1920x1080
export DEMO_FPS=60
DEMO_ALLOW_CLICKHOUSE_RESET=true python scripts/demo/run_human_rehearsal.py --no-voice
```

Outputs land under `demo-artifacts/review-human/`, including the no-audio
review video, subtitle sidecars, the burned-in subtitled review video, and the
browser-viewable subtitle alignment timeline.

The recorder disables mouse capture, so the cursor should not appear in the
review videos.

If you already have a current no-audio review video and only need to regenerate
subtitle sidecars plus the burned-in subtitle render, run:

```sh
source .venv-demo/bin/activate
python scripts/demo/subtitles.py
```

Current human-scene flow:

- `scene-01-readiness-checks`: terminal health checks prove ClickHouse,
  Prometheus, Grafana, and RPC reachability are online.
- `scene-02-example-env`: VS Code shows both example `.env.example` files and
  enables async-wait inserts in each example configuration.
- `scene-03-jupiter-live`: terminal starts the Jupiter live example with
  async-wait inserts.
- `scene-04-token-live`: terminal starts the Token Program example with
  async-wait inserts.
- `scene-05-observability`: filtered Jupiter and Token Program Grafana
  dashboards show the running Carbon pipelines, then ClickStack shows
  ClickHouse-side insert rows and bytes per table.
- `scene-06-clickhouse-play`: ClickHouse `/play` uses the table browser,
  per-example table-family grouping, top populated landing tables, and the
  largest populated landing table to verify generated tables, data volume, and
  queryable rows.

Review the subtitle and screen-state alignment in:

```sh
demo-artifacts/review-human/subtitle-alignment-timeline.html
```

Current theme behavior:

- VS Code uses a dedicated recording profile outside the repository.
- ClickHouse `/play` and ClickStack are darkened through deterministic
  recording-time CSS injection without decorative backgrounds.
- Grafana uses its native dark theme/dashboard URL and is zoomed to 80% so the
  lower dashboard panels fit in the recording frame.
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

For clean tutorial recording, use a dedicated recording display and source its
display env file before launching the rehearsal:

```sh
source "$DEMO_RECORDING_DISPLAY_ENV"
export DEMO_SCREEN_SIZE=1920x1080
```

Before deterministic tutorial runs, reset tutorial ClickHouse tables:

```sh
DEMO_ALLOW_CLICKHOUSE_RESET=true scripts/demo/reset-clickhouse.sh
```

Voice generation is selected with `TTS_PROVIDER` in `.env.demo.local`.
ElevenLabs is the default and requires an API key. A voice ID can be set
manually, or `voice.py` can discover a default/free-usable voice and write it
without printing the ID:

```env
TTS_PROVIDER=elevenlabs
ELEVENLABS_API_KEY=<elevenlabs-api-key>
ELEVENLABS_VOICE_ID=<voice-id>
ELEVENLABS_MODEL_ID=eleven_multilingual_v2
ELEVENLABS_OUTPUT_FORMAT=mp3_44100_128
ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK=false
ELEVENLABS_AUTO_WRITE_VOICE_ID=false
```

Gemini TTS uses the Google AI Studio/Gemini API key and writes WAV audio. Use a
Gemini TTS-capable model here; text-generation aliases such as
`gemini-flash-latest` are not valid for voiceover generation. Gemini returns
raw audio with MIME metadata, and the helper wraps it as WAV using the returned
sample rate and bit depth.

```env
TTS_PROVIDER=gemini
GEMINI_API_KEY=<gemini-api-key>
GEMINI_TTS_MODEL=gemini-3.1-flash-tts-preview
GEMINI_TTS_VOICE=Iapetus
GEMINI_TTS_SAMPLE_RATE=24000
GEMINI_TTS_TEMPERATURE=1
GEMINI_TTS_PROMPT_STYLE=software-tutorial
GEMINI_TTS_PROMPT_PREFIX_FILE=
```

`software-tutorial` prompt style wraps each scene under a `# TRANSCRIPT`
section with a small tag set for instruction, explanation, confirmation, and
short pauses. Use `GEMINI_TTS_PROMPT_PREFIX_FILE` when you want to override the
built-in audio profile and director note.

Alibaba Cloud TTS uses Model Studio DashScope with CosyVoice. The non-streaming
SDK call accepts up to 20,000 characters, so the helper chunks longer scene
scripts and concatenates the returned audio:

```env
TTS_PROVIDER=alibaba
DASHSCOPE_API_KEY=<dashscope-api-key>
DASHSCOPE_BASE_WEBSOCKET_API_URL=wss://dashscope-intl.aliyuncs.com/api-ws/v1/inference
ALIBABA_DASHSCOPE_MODEL=cosyvoice-v3-plus
ALIBABA_DASHSCOPE_VOICE=longanyang
ALIBABA_DASHSCOPE_AUDIO_FORMAT=
ALIBABA_DASHSCOPE_MAX_CHARS=20000
```

Leave `ALIBABA_DASHSCOPE_AUDIO_FORMAT` empty for DashScope's default MP3
output, or set it to a DashScope `AudioFormat` enum name such as
`MP3_22050HZ_MONO_256KBPS` or `WAV_24000HZ_MONO_16BIT`.

```sh
source .venv-demo/bin/activate
python scripts/demo/voice.py status
python scripts/demo/voice.py check
python scripts/demo/voice.py smoke
```

For ElevenLabs only, run `python scripts/demo/voice.py select-free --write-env`
before the smoke test if you want the helper to choose and store a free-usable
voice ID.

OBS backup recording is valid only after:

```sh
scripts/demo/start-obs.sh
RECORDER_BACKEND=obs python scripts/demo/record.py status
RECORDER_BACKEND=obs python scripts/demo/record.py start --scene obs-smoke
RECORDER_BACKEND=obs python scripts/demo/record.py stop --scene obs-smoke
scripts/demo/stop-obs.sh
```
