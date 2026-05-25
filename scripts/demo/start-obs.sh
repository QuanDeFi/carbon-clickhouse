#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env
ensure_demo_dirs

DISPLAY_ID="${DEMO_DISPLAY:-:95}"
SCREEN_SIZE="${DEMO_SCREEN_SIZE:-1920x1080}"
DEPTH="${DEMO_SCREEN_DEPTH:-24}"
OBS_PORT="${OBS_WEBSOCKET_PORT:-4455}"
OBS_UNIT="${DEMO_OBS_UNIT:-carbon-demo-obs}"
XVFB_UNIT="${DEMO_OBS_XVFB_UNIT:-carbon-demo-obs-xvfb}"
DISPLAY_ENV="$ROOT/demo-artifacts/display.env"

display_number() {
  printf '%s\n' "${1#:}" | cut -d. -f1
}

display_reachable() {
  xdpyinfo -display "$1" >/dev/null 2>&1
}

cleanup_stale_display_files() {
  local display="$1"
  local number
  number="$(display_number "$display")"
  if [[ -e "/tmp/.X$number-lock" || -S "/tmp/.X11-unix/X$number" ]] \
    && ! display_reachable "$display" \
    && ! pgrep -f "Xvfb $display" >/dev/null; then
    log_warn "Removing stale X display files for $display"
    rm -f "/tmp/.X$number-lock" "/tmp/.X11-unix/X$number"
  fi
}

set_obs_websocket_config() {
  local config="$HOME/.config/obs-studio/global.ini"
  mkdir -p "$(dirname "$config")"
  if [[ ! -f "$config" ]]; then
    cat > "$config" <<'EOF'
[General]
FirstRun=false

[OBSWebSocket]
FirstLoad=false
ServerEnabled=true
ServerPort=4455
AlertsEnabled=false
AuthRequired=false
EOF
  fi

  "$ROOT/.venv-demo/bin/python" - "$config" "$OBS_PORT" "${OBS_WEBSOCKET_PASSWORD:-}" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
port = sys.argv[2]
password = sys.argv[3]
lines = path.read_text(errors="ignore").splitlines()
out = []
in_section = False
seen_section = False
seen = set()
settings = {
    "FirstLoad": "false",
    "ServerEnabled": "true",
    "ServerPort": port,
    "AlertsEnabled": "false",
    "AuthRequired": "true" if password else "false",
}
if password:
    settings["ServerPassword"] = password

for line in lines:
    stripped = line.strip()
    if stripped.startswith("[") and stripped.endswith("]"):
        if in_section:
            for key, value in settings.items():
                if key not in seen:
                    out.append(f"{key}={value}")
        in_section = stripped == "[OBSWebSocket]"
        seen_section = seen_section or in_section
        seen = set() if in_section else seen
        out.append(line)
        continue
    if in_section and "=" in line:
        key = line.split("=", 1)[0].strip()
        if key in settings:
            out.append(f"{key}={settings[key]}")
            seen.add(key)
            continue
    out.append(line)

if in_section:
    for key, value in settings.items():
        if key not in seen:
            out.append(f"{key}={value}")
elif not seen_section:
    out.extend(["", "[OBSWebSocket]"])
    out.extend(f"{key}={value}" for key, value in settings.items())

path.write_text("\n".join(out) + "\n")
path.chmod(0o600)
PY
}

wait_for_display() {
  for _ in {1..50}; do
    display_reachable "$DISPLAY_ID" && return 0
    sleep 0.2
  done
  return 1
}

wait_for_websocket() {
  for _ in {1..80}; do
    if timeout 1 bash -c "</dev/tcp/127.0.0.1/$OBS_PORT" 2>/dev/null; then
      return 0
    fi
    sleep 0.25
  done
  return 1
}

cleanup_stale_display_files "$DISPLAY_ID"
set_obs_websocket_config

if ! display_reachable "$DISPLAY_ID"; then
  log_info "Starting OBS Xvfb unit $XVFB_UNIT on $DISPLAY_ID"
  systemctl --user stop "$XVFB_UNIT.service" >/dev/null 2>&1 || true
  systemd-run --user --unit="$XVFB_UNIT" --collect \
    /usr/bin/Xvfb "$DISPLAY_ID" -screen 0 "${SCREEN_SIZE}x${DEPTH}" -ac -noreset >/dev/null
  wait_for_display || { log_error "Display $DISPLAY_ID did not become reachable"; exit 1; }
fi

if ! pgrep -x obs >/dev/null; then
  log_info "Starting OBS unit $OBS_UNIT"
  systemctl --user stop "$OBS_UNIT.service" >/dev/null 2>&1 || true
  systemd-run --user --unit="$OBS_UNIT" \
    --setenv=DISPLAY="$DISPLAY_ID" \
    --setenv=QT_QPA_PLATFORM=xcb \
    --collect \
    /usr/bin/obs --disable-shutdown-check --multi --verbose >/dev/null
fi

wait_for_websocket || { log_error "OBS websocket did not become reachable on port $OBS_PORT"; exit 1; }

cat > "$DISPLAY_ENV" <<EOF
DEMO_DISPLAY=$DISPLAY_ID
DISPLAY=$DISPLAY_ID
DEMO_SCREEN_SIZE=$SCREEN_SIZE
DEMO_VNC_PORT=${DEMO_VNC_PORT:-5903}
DEMO_NOVNC_PORT=${DEMO_NOVNC_PORT:-6083}
DEMO_NOVNC_URL=http://localhost:${DEMO_NOVNC_PORT:-6083}/vnc.html
EOF

systemctl --user show "$OBS_UNIT.service" -p MainPID --value > "$ROOT/demo-artifacts/pids/obs.pid" 2>/dev/null || true
systemctl --user show "$XVFB_UNIT.service" -p MainPID --value > "$ROOT/demo-artifacts/pids/obs-xvfb.pid" 2>/dev/null || true

cat <<EOF
OBS websocket: reachable
OBS unit: $OBS_UNIT.service
Display: $DISPLAY_ID
EOF
