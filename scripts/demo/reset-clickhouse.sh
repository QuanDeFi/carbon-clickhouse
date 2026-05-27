#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env
ensure_demo_dirs
RESET_LOG="$ROOT/demo-artifacts/logs/clickhouse-reset.log"
exec > >(tee "$RESET_LOG") 2>&1

DATABASE_URL="${DATABASE_URL:-http://carbon:carbon@localhost:8123}"

query_ch() {
  curl -fsS "$DATABASE_URL/" --data-binary "$1" | mask_secrets
}

active_demo_processes() {
  # grep (not rg) so the reset works on hosts without ripgrep installed.
  pgrep -af 'jupiter-swap-clickhouse-carbon-example|token-program-clickhouse-carbon-example|cargo run -p jupiter-swap-clickhouse-carbon-example|cargo run -p token-program-clickhouse-carbon-example' \
    | grep -Ev 'pgrep|reset-clickhouse|detect-running|bash -c' || true
}

# System logs surfaced to viewers in ClickStack. They accumulate across runs
# (e.g. system.query_log grows unbounded), so each demo run otherwise shows
# stale prior-run telemetry. Truncated for a fresh viewer-visible state.
SYSTEM_LOGS=(system.query_log system.asynchronous_insert_log system.metric_log)

# Refuse to reset while example processes are still writing.
running="$(active_demo_processes)"
if [[ -n "$running" ]]; then
  log_error "Demo/example processes are active; stop them before reset"
  printf '%s\n' "$running" | mask_secrets
  exit 1
fi

tables="$(curl -fsS "$DATABASE_URL/" --data-binary "SHOW TABLES FROM default" \
  | awk '/^jupiter_swap_.*landing$/ || /^token_program_.*account_landing$/ {print}' \
  | sort)"

if [[ "${DEMO_ALLOW_CLICKHOUSE_RESET:-}" != "true" ]]; then
  log_warn "Dry run only. Set DEMO_ALLOW_CLICKHOUSE_RESET=true to drop tutorial tables and flush system logs."
  if [[ -n "$tables" ]]; then
    log_info "Would drop tutorial tables:"
    printf '%s\n' "$tables"
  fi
  exit 0
fi

if [[ -n "$tables" ]]; then
  log_info "Dropping tutorial tables:"
  printf '%s\n' "$tables"
  while IFS= read -r table; do
    [[ -z "$table" ]] && continue
    log_info "Dropping $table"
    query_ch "DROP TABLE IF EXISTS default.$table"
  done <<< "$tables"
else
  log_info "No tutorial ClickHouse tables matched (nothing to drop)"
fi

log_info "Flushing ClickHouse system logs for a fresh viewer-visible state"
for system_log in "${SYSTEM_LOGS[@]}"; do
  if query_ch "TRUNCATE TABLE IF EXISTS $system_log"; then
    log_info "Truncated $system_log"
  else
    log_warn "Could not truncate $system_log (continuing)"
  fi
done
