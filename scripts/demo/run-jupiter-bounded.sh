#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env
ensure_demo_dirs

require_env SOLANA_RPC_URL
require_env JUPITER_START_SLOT
require_env JUPITER_END_SLOT

if port_listener 9464 >/dev/null; then
  log_error "port 9464 is already occupied; stop the existing Jupiter metrics process first"
  port_listener 9464 | mask_secrets
  exit 1
fi

LOG_FILE="$ROOT/demo-artifacts/logs/jupiter-bounded-$(date +%Y%m%d-%H%M%S).log"
log_info "Writing masked log to $LOG_FILE"

cd "$ROOT"
export DATABASE_URL="${DATABASE_URL:-http://carbon:carbon@localhost:8123}"
export RPC_URL="$SOLANA_RPC_URL"
export BLOCK_CRAWLER_START_SLOT="$JUPITER_START_SLOT"
export BLOCK_CRAWLER_END_SLOT="$JUPITER_END_SLOT"
export BLOCK_CRAWLER_HEAD_LAG_SLOTS="${BLOCK_CRAWLER_HEAD_LAG_SLOTS:-3}"
export PROMETHEUS_METRICS_ADDR="${JUPITER_PROMETHEUS_METRICS_ADDR:-0.0.0.0:9464}"
export LOG_LEVEL="${LOG_LEVEL:-info}"

set +e
cargo run -p jupiter-swap-clickhouse-carbon-example 2>&1 \
  | "$ROOT/scripts/demo/mask_output.py" \
  | tee "$LOG_FILE"
status=${PIPESTATUS[0]}
set -e
exit "$status"
