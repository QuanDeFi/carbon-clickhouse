#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env
ensure_demo_dirs

AUTO=false
for arg in "$@"; do
  case "$arg" in
    --auto) AUTO=true ;;
    *) log_error "Unknown start-display flag: $arg"; exit 2 ;;
  esac
done

DISPLAY_ID="${DEMO_DISPLAY:-:96}"
SCREEN_SIZE="${DEMO_SCREEN_SIZE:-1920x1080}"
DEPTH="${DEMO_SCREEN_DEPTH:-24}"
VNC_PORT="${DEMO_VNC_PORT:-5904}"
NOVNC_PORT="${DEMO_NOVNC_PORT:-6084}"
PID_DIR="$ROOT/demo-artifacts/pids"
LOG_DIR="$ROOT/demo-artifacts/logs"
DISPLAY_ENV="$ROOT/demo-artifacts/display.env"
SKIP_NOVNC="${DEMO_SKIP_NOVNC:-false}"

display_number() {
  printf '%s\n' "${1#:}" | cut -d. -f1
}

display_occupied() {
  local number
  number="$(display_number "$1")"
  [[ -S "/tmp/.X11-unix/X$number" || -e "/tmp/.X$number-lock" ]]
}

display_reachable() {
  xdpyinfo -display "$1" >/dev/null 2>&1
}

cleanup_stale_display_files() {
  local display="$1"
  local number
  number="$(display_number "$display")"
  if display_occupied "$display" && ! display_reachable "$display" && ! pgrep -f "Xvfb $display" >/dev/null; then
    log_warn "Removing stale X display files for $display"
    rm -f "/tmp/.X$number-lock" "/tmp/.X11-unix/X$number"
  fi
}

choose_free_display() {
  local number
  number="$(display_number "$DISPLAY_ID")"
  for candidate in $(seq "$number" $((number + 20))); do
    if ! display_occupied ":$candidate"; then
      DISPLAY_ID=":$candidate"
      return 0
    fi
  done
  return 1
}

choose_free_port() {
  local var_name="$1"
  local port="${!var_name}"
  for candidate in $(seq "$port" $((port + 50))); do
    if ! port_listener "$candidate" >/dev/null; then
      printf -v "$var_name" '%s' "$candidate"
      return 0
    fi
  done
  return 1
}

start_cmd() {
  local name="$1"
  local pid_file="$PID_DIR/$name.pid"
  local log_file="$LOG_DIR/$name.log"
  shift
  if [[ -f "$pid_file" ]] && kill -0 "$(cat "$pid_file")" 2>/dev/null; then
    log_info "$name already running with PID $(cat "$pid_file")"
    return 0
  fi
  log_info "Starting $name"
  rm -f "$pid_file"
  setsid -f bash -c 'echo $$ > "$1"; shift; exec "$@"' _ "$pid_file" "$@" > "$log_file" 2>&1 < /dev/null
  for _ in {1..20}; do
    [[ -s "$pid_file" ]] && break
    sleep 0.1
  done
  if [[ ! -s "$pid_file" ]]; then
    log_error "$name failed to write a PID file; see $log_file"
    return 1
  fi
  sleep 1
  if ! kill -0 "$(cat "$pid_file")" 2>/dev/null; then
    log_error "$name failed to start; see $log_file"
    return 1
  fi
}

cleanup_stale_display_files "$DISPLAY_ID"

if display_occupied "$DISPLAY_ID" && [[ ! -f "$PID_DIR/xvfb.pid" ]]; then
  if [[ "$AUTO" == true ]]; then
    choose_free_display || { log_error "No free X display found near $DISPLAY_ID"; exit 1; }
  else
    log_error "Display $DISPLAY_ID is already in use by an unmanaged process; use --auto or set DEMO_DISPLAY"
    exit 1
  fi
fi

if [[ "$SKIP_NOVNC" != "true" ]] && port_listener "$VNC_PORT" >/dev/null && [[ ! -f "$PID_DIR/x11vnc.pid" ]]; then
  if [[ "$AUTO" == true ]]; then
    choose_free_port VNC_PORT || { log_error "No free VNC port found near $VNC_PORT"; exit 1; }
  else
    log_error "Port $VNC_PORT is already in use by an unmanaged process"
    port_listener "$VNC_PORT"
    exit 1
  fi
fi
if [[ "$SKIP_NOVNC" != "true" ]] && port_listener "$NOVNC_PORT" >/dev/null && [[ ! -f "$PID_DIR/websockify.pid" ]]; then
  if [[ "$AUTO" == true ]]; then
    choose_free_port NOVNC_PORT || { log_error "No free noVNC port found near $NOVNC_PORT"; exit 1; }
  else
    log_error "Port $NOVNC_PORT is already in use by an unmanaged process"
    port_listener "$NOVNC_PORT"
    exit 1
  fi
fi

start_cmd xvfb Xvfb "$DISPLAY_ID" -screen 0 "${SCREEN_SIZE}x${DEPTH}" -ac -noreset
if [[ "$SKIP_NOVNC" == "true" ]]; then
  log_info "Skipping x11vnc/noVNC sidecars because DEMO_SKIP_NOVNC=true"
  rm -f "$PID_DIR/x11vnc.pid" "$PID_DIR/websockify.pid"
else
  start_cmd x11vnc x11vnc -display "$DISPLAY_ID" -forever -shared -rfbport "$VNC_PORT" -nopw
  start_cmd websockify websockify --web=/usr/share/novnc/ "$NOVNC_PORT" "localhost:$VNC_PORT"
fi

cat > "$DISPLAY_ENV" <<EOF
DEMO_DISPLAY=$DISPLAY_ID
DISPLAY=$DISPLAY_ID
DEMO_SCREEN_SIZE=$SCREEN_SIZE
DEMO_VNC_PORT=$VNC_PORT
DEMO_NOVNC_PORT=$NOVNC_PORT
DEMO_SKIP_NOVNC=$SKIP_NOVNC
DEMO_NOVNC_URL=http://localhost:$NOVNC_PORT/vnc.html
EOF

cat <<EOF
DISPLAY=$DISPLAY_ID
noVNC: $(if [[ "$SKIP_NOVNC" == "true" ]]; then printf 'disabled'; else printf 'http://localhost:%s/vnc.html' "$NOVNC_PORT"; fi)
EOF
