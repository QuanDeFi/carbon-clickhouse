#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env
ensure_demo_dirs

require_env SOLANA_RPC_URL

if port_listener 9465 >/dev/null; then
  log_error "port 9465 is already occupied; stop the existing Token metrics process first"
  port_listener 9465 | mask_secrets
  exit 1
fi

LOG_FILE="$ROOT/demo-artifacts/logs/token-snapshot-$(date +%Y%m%d-%H%M%S).log"
METRICS_FILE="$ROOT/demo-artifacts/logs/token-metrics-$(date +%Y%m%d-%H%M%S).txt"
log_info "Writing masked log to $LOG_FILE"

cd "$ROOT"
export DATABASE_URL="${DATABASE_URL:-http://carbon:carbon@localhost:8123}"
export RPC_URL="$SOLANA_RPC_URL"
export PROMETHEUS_METRICS_ADDR="${TOKEN_PROMETHEUS_METRICS_ADDR:-0.0.0.0:9465}"
export LOG_LEVEL="${LOG_LEVEL:-info}"

set +e
cargo run -p token-program-clickhouse-carbon-example 2>&1 \
  | "$ROOT/scripts/demo/mask_output.py" \
  | tee "$LOG_FILE" &
run_pid=$!

for _ in {1..30}; do
  if curl -fsS "http://localhost:${PROMETHEUS_METRICS_ADDR##*:}/metrics" > "$METRICS_FILE" 2>/dev/null; then
    "$ROOT/scripts/demo/mask_output.py" < "$METRICS_FILE" > "$METRICS_FILE.masked"
    mv "$METRICS_FILE.masked" "$METRICS_FILE"
    log_info "Captured token metrics to $METRICS_FILE"
    break
  fi
  sleep 0.2
done

wait "$run_pid"
status=$?
set -e
exit "$status"
