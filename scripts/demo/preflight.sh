#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env

CHECK_OBS=true
CHECK_ELEVENLABS=true
CHECK_BROWSER=true
FULL_RECORDING=false
for arg in "$@"; do
  case "$arg" in
    --no-obs) CHECK_OBS=false ;;
    --no-elevenlabs) CHECK_ELEVENLABS=false ;;
    --terminal-only) CHECK_BROWSER=false ;;
    --full-recording) FULL_RECORDING=true ;;
    *) log_error "Unknown preflight flag: $arg"; exit 2 ;;
  esac
done

if [[ "$FULL_RECORDING" == true ]]; then
  CHECK_ELEVENLABS=true
fi

failures=0
warnings=0

ok() { printf '[ok] %s\n' "$*"; }
warn() { printf '[warn] %s\n' "$*"; warnings=$((warnings + 1)); }
fail() { printf '[fail] %s\n' "$*"; failures=$((failures + 1)); }

require_cmd() {
  command -v "$1" >/dev/null 2>&1 && ok "$1 present" || fail "$1 missing"
}

display_number() {
  printf '%s\n' "${1#:}" | cut -d. -f1
}

display_socket_exists() {
  local number
  number="$(display_number "$1")"
  [[ -S "/tmp/.X11-unix/X$number" || -e "/tmp/.X$number-lock" ]]
}

display_stale() {
  local display="$1"
  display_socket_exists "$display" && ! xdpyinfo -display "$display" >/dev/null 2>&1 && ! pgrep -f "Xvfb $display" >/dev/null
}

display_matches_size() {
  local display="$1"
  local size="$2"
  xdpyinfo -display "$display" 2>/dev/null | grep -q "dimensions:    $size pixels"
}

ffmpeg_x11_smoke() {
  local display="$1"
  local size="$2"
  for _ in 1 2 3; do
    if timeout 5 ffmpeg -hide_banner -loglevel error -y \
      -video_size "$size" \
      -framerate 1 \
      -f x11grab \
      -i "$display" \
      -frames:v 1 \
      -f null - >/dev/null 2>&1; then
      return 0
    fi
    sleep 0.5
  done
  return 1
}

cd "$ROOT"
[[ "$(git rev-parse --show-toplevel)" == "$ROOT" ]] && ok "repo root $ROOT" || fail "not in repo root"
branch="$(git branch --show-current)"
[[ "$branch" == "clickhouse-upstream-v1" ]] && ok "branch $branch" || warn "branch is $branch"

if git rev-parse --verify origin/clickhouse-upstream-v1 >/dev/null 2>&1; then
  read -r ahead behind < <(git rev-list --left-right --count origin/clickhouse-upstream-v1...HEAD)
  if [[ "$ahead" == "0" && "$behind" == "0" ]]; then
    ok "origin/clickhouse-upstream-v1 aligned"
  else
    warn "origin/clickhouse-upstream-v1 ahead/behind relative to HEAD: $ahead/$behind"
  fi
else
  warn "origin/clickhouse-upstream-v1 unavailable"
fi

dirty="$(git status --short)"
if [[ -n "$dirty" ]]; then
  if [[ "$dirty" == ' M "crates/core/src/clickhouse/docs/Branch Diff to v1.md"' || "$dirty" == " M crates/core/src/clickhouse/docs/Branch Diff to v1.md" ]]; then
    warn "only Branch Diff to v1.md is dirty"
  else
    warn "working tree is dirty"
    printf '%s\n' "$dirty"
  fi
else
  ok "working tree clean"
fi

for cmd in cargo rustc docker curl jq ffmpeg xterm Xvfb x11vnc websockify python3 xdpyinfo; do
  require_cmd "$cmd"
done
docker compose version >/dev/null 2>&1 && ok "docker compose present" || fail "docker compose unavailable"

curl -fsS 'http://carbon:carbon@localhost:8123/?query=SELECT%201' >/dev/null && ok "ClickHouse SELECT 1" || fail "ClickHouse unavailable"
curl -fsS 'http://localhost:9090/-/ready' >/dev/null && ok "Prometheus ready" || fail "Prometheus unavailable"
curl -fsS 'http://localhost:3000/api/health' >/dev/null && ok "Grafana health" || fail "Grafana unavailable"

if [[ -x "$ROOT/.venv-demo/bin/python" ]]; then
  ok "Python venv present"
  if "$ROOT/.venv-demo/bin/python" - <<'PY'
import importlib.util
mods = ["yaml", "requests", "dotenv", "obsws_python", "websocket", "rich"]
missing = [m for m in mods if importlib.util.find_spec(m) is None]
if missing:
    print("[fail] missing venv packages: " + ", ".join(missing))
    raise SystemExit(1)
print("[ok] Python venv packages present")
PY
  then
    :
  else
    failures=$((failures + 1))
  fi
else
  fail "Python venv .venv-demo missing"
fi

if [[ "$CHECK_BROWSER" == true ]]; then
  if [[ -d "$ROOT/scripts/demo/playwright/node_modules/@playwright/test" ]]; then
    ok "demo-local Playwright package present"
  elif npx playwright --version >/dev/null 2>&1; then
    warn "demo-local Playwright package missing; npx playwright is available"
  else
    fail "Playwright unavailable"
  fi
fi

require_env SOLANA_RPC_URL || failures=$((failures + 1))
require_env JUPITER_START_SLOT || failures=$((failures + 1))
require_env JUPITER_END_SLOT || failures=$((failures + 1))

DISPLAY_ID="${DEMO_DISPLAY:-:95}"
SCREEN_SIZE="${DEMO_SCREEN_SIZE:-1920x1080}"
VNC_PORT="${DEMO_VNC_PORT:-5903}"
NOVNC_PORT="${DEMO_NOVNC_PORT:-6083}"

[[ "$DISPLAY_ID" == :95 ]] && ok "clean display default $DISPLAY_ID" || warn "selected display is $DISPLAY_ID"
[[ "$SCREEN_SIZE" == "1920x1080" ]] && ok "screen size $SCREEN_SIZE" || warn "screen size is $SCREEN_SIZE"
[[ "$VNC_PORT" == "5903" ]] && ok "VNC port $VNC_PORT" || warn "VNC port is $VNC_PORT"
[[ "$NOVNC_PORT" == "6083" ]] && ok "noVNC port $NOVNC_PORT" || warn "noVNC port is $NOVNC_PORT"

if display_stale "$DISPLAY_ID"; then
  warn "display $DISPLAY_ID has stale X socket/lock files; start-display will clean them"
  if port_listener "$VNC_PORT" >/dev/null; then
    fail "VNC port $VNC_PORT is occupied"
    port_listener "$VNC_PORT"
  else
    ok "VNC port $VNC_PORT is free"
  fi
  if port_listener "$NOVNC_PORT" >/dev/null; then
    fail "noVNC port $NOVNC_PORT is occupied"
    port_listener "$NOVNC_PORT"
  else
    ok "noVNC port $NOVNC_PORT is free"
  fi
elif display_socket_exists "$DISPLAY_ID"; then
  if display_matches_size "$DISPLAY_ID" "$SCREEN_SIZE"; then
    ok "display $DISPLAY_ID is active at $SCREEN_SIZE"
    if ffmpeg_x11_smoke "$DISPLAY_ID" "$SCREEN_SIZE"; then
      ok "FFmpeg x11grab smoke passed on $DISPLAY_ID"
    else
      warn "FFmpeg x11grab smoke failed on $DISPLAY_ID; display is reachable, so continuing"
    fi
  else
    fail "display $DISPLAY_ID is occupied but not reachable at $SCREEN_SIZE"
  fi
else
  ok "display $DISPLAY_ID is free"
  if port_listener "$VNC_PORT" >/dev/null; then
    fail "VNC port $VNC_PORT is occupied"
    port_listener "$VNC_PORT"
  else
    ok "VNC port $VNC_PORT is free"
  fi
  if port_listener "$NOVNC_PORT" >/dev/null; then
    fail "noVNC port $NOVNC_PORT is occupied"
    port_listener "$NOVNC_PORT"
  else
    ok "noVNC port $NOVNC_PORT is free"
  fi
fi

if [[ "$CHECK_ELEVENLABS" == true ]]; then
  require_env ELEVENLABS_API_KEY || failures=$((failures + 1))
  if require_env ELEVENLABS_VOICE_ID; then
    :
  elif [[ "${ELEVENLABS_DISABLE_AUTO_VOICE_FALLBACK:-false}" =~ ^(1|true|yes|on)$ ]]; then
    failures=$((failures + 1))
  else
    warn "ELEVENLABS_VOICE_ID missing; voice.py will try a default/free-usable voice"
  fi
else
  warn "Skipping ElevenLabs checks"
fi

if port_listener 9464 >/dev/null; then
  warn "port 9464 is occupied; deterministic Jupiter recording may conflict"
  port_listener 9464
else
  ok "port 9464 free"
fi
if port_listener 9465 >/dev/null; then
  warn "port 9465 is occupied; deterministic Token recording may conflict"
  port_listener 9465
else
  ok "port 9465 free"
fi

if pgrep -af 'jupiter-swap-clickhouse-carbon-example|token-program-clickhouse-carbon-example|cargo run -p jupiter-swap-clickhouse-carbon-example|cargo run -p token-program-clickhouse-carbon-example' | grep -Ev 'pgrep|preflight|detect-running|bash -c' >/dev/null; then
  warn "demo/example process appears active"
else
  ok "no active demo/example process detected"
fi

if command -v obs >/dev/null 2>&1; then
  ok "OBS installed"
else
  warn "OBS missing"
fi

if [[ "${RECORDER_BACKEND:-ffmpeg_x11}" == "obs" && "$CHECK_OBS" == true ]]; then
  if timeout 1 bash -c "</dev/tcp/${OBS_WEBSOCKET_HOST:-127.0.0.1}/${OBS_WEBSOCKET_PORT:-4455}" 2>/dev/null; then
    ok "OBS websocket reachable"
  else
    fail "OBS websocket unavailable"
  fi
elif [[ "$CHECK_OBS" == false ]]; then
  warn "Skipping OBS websocket checks"
fi

if [[ "$FULL_RECORDING" == true ]]; then
  if [[ "${RECORDER_BACKEND:-ffmpeg_x11}" == "ffmpeg_x11" && ! -S "/tmp/.X11-unix/X$(display_number "$DISPLAY_ID")" ]]; then
    fail "--full-recording requires an active display for ffmpeg_x11"
  fi
  smoke_file="$ROOT/demo-artifacts/audio/elevenlabs-smoke.mp3"
  if [[ -s "$smoke_file" && $(find "$smoke_file" -mmin -1440 -print) ]]; then
    ok "recent ElevenLabs smoke audio present"
  elif "$ROOT/.venv-demo/bin/python" "$ROOT/scripts/demo/voice.py" smoke >/dev/null; then
    ok "ElevenLabs smoke succeeded"
  else
    fail "ElevenLabs smoke failed"
  fi
  if [[ -s "$ROOT/demo-artifacts/logs/clickhouse-reset.log" || "${DEMO_CLICKHOUSE_RESET_ACK:-}" == "true" ]]; then
    ok "ClickHouse reset acknowledged"
  else
    fail "--full-recording requires clickhouse-reset.log or DEMO_CLICKHOUSE_RESET_ACK=true"
  fi
fi

printf '\nPreflight: %s failure(s), %s warning(s)\n' "$failures" "$warnings"
[[ "$failures" -eq 0 ]]
