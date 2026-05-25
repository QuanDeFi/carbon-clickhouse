#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env

OBS_UNIT="${DEMO_OBS_UNIT:-carbon-demo-obs}"
XVFB_UNIT="${DEMO_OBS_XVFB_UNIT:-carbon-demo-obs-xvfb}"

log_info "Stopping OBS unit $OBS_UNIT.service"
systemctl --user stop "$OBS_UNIT.service" >/dev/null 2>&1 || true

log_info "Stopping OBS Xvfb unit $XVFB_UNIT.service"
systemctl --user stop "$XVFB_UNIT.service" >/dev/null 2>&1 || true

rm -f "$ROOT/demo-artifacts/pids/obs.pid" "$ROOT/demo-artifacts/pids/obs-xvfb.pid"
