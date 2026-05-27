#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"

PID_DIR="$ROOT/demo-artifacts/pids"

stop_pid() {
  local name="$1"
  local pid_file="$PID_DIR/$name.pid"
  if [[ ! -f "$pid_file" ]]; then
    log_info "$name pid file not present"
    return 0
  fi
  local pid
  pid="$(cat "$pid_file")"
  if kill -0 "$pid" 2>/dev/null; then
    log_info "Stopping $name PID $pid"
    kill "$pid" 2>/dev/null || true
    for _ in {1..20}; do
      kill -0 "$pid" 2>/dev/null || break
      sleep 0.2
    done
    kill -0 "$pid" 2>/dev/null && kill -9 "$pid" 2>/dev/null || true
  fi
  rm -f "$pid_file"
}

stop_pid websockify
stop_pid x11vnc
for browser_pid_file in "$PID_DIR"/browser-*.pid; do
  [[ -e "$browser_pid_file" ]] || continue
  browser_name="$(basename "$browser_pid_file" .pid)"
  stop_pid "$browser_name"
done
stop_pid xvfb
rm -f "$ROOT/demo-artifacts/display.env"
