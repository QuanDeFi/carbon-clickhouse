#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"

mapfile -t PIDS < <(pgrep -f 'jupiter-swap-clickhouse-carbon-example|token-program-clickhouse-carbon-example|cargo run -p jupiter-swap-clickhouse-carbon-example|cargo run -p token-program-clickhouse-carbon-example' || true)

if [[ "${DEMO_ALLOW_STOP_PROCESSES:-}" != "true" ]]; then
  log_warn "DEMO_ALLOW_STOP_PROCESSES=true is required to stop demo processes"
  "$ROOT/scripts/demo/detect-running-demo-processes.sh"
  exit 0
fi

for unit in \
  carbon-jupiter-swap-clickhouse-live.service \
  carbon-token-program-clickhouse.service \
  carbon-jupiter-clickhouse-backfill.service
do
  if systemctl --user is-active --quiet "$unit"; then
    log_info "Stopping user service $unit"
    systemctl --user stop "$unit"
  fi
done

for pid in "${PIDS[@]}"; do
  if [[ "$pid" == "$$" ]]; then
    continue
  fi
  if kill -0 "$pid" 2>/dev/null; then
    log_info "Stopping PID $pid"
    kill "$pid" 2>/dev/null || true
  fi
done
