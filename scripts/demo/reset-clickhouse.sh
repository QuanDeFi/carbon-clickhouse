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
  pgrep -af 'jupiter-swap-clickhouse-carbon-example|token-program-clickhouse-carbon-example|cargo run -p jupiter-swap-clickhouse-carbon-example|cargo run -p token-program-clickhouse-carbon-example' \
    | rg -v 'pgrep|reset-clickhouse|detect-running|bash -c' || true
}

tables="$(curl -fsS "$DATABASE_URL/" --data-binary "SHOW TABLES FROM default" \
  | awk '/^jupiter_swap_.*landing$/ || /^token_program_.*account_landing$/ {print}' \
  | sort)"

if [[ -z "$tables" ]]; then
  log_info "No tutorial ClickHouse tables matched"
  exit 0
fi

log_info "Matching tutorial tables:"
printf '%s\n' "$tables"

running="$(active_demo_processes)"
if [[ -n "$running" ]]; then
  log_error "Demo/example processes are active; stop them before reset"
  printf '%s\n' "$running" | mask_secrets
  exit 1
fi

if [[ "${DEMO_ALLOW_CLICKHOUSE_RESET:-}" != "true" ]]; then
  log_warn "Dry run only. Set DEMO_ALLOW_CLICKHOUSE_RESET=true to drop these tables."
  exit 0
fi

while IFS= read -r table; do
  [[ -z "$table" ]] && continue
  log_info "Dropping $table"
  query_ch "DROP TABLE IF EXISTS default.$table"
done <<< "$tables"
