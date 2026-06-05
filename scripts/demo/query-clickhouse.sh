#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env

DATABASE_URL="${DATABASE_URL:-http://carbon:carbon@localhost:8123}"
command="${1:-}"

run_query() {
  curl -fsS "$DATABASE_URL/" --data-binary "$1" | mask_secrets
}

case "$command" in
  health)
    run_query "SELECT 1"
    ;;
  jupiter)
    run_query "SHOW TABLES FROM default LIKE 'jupiter_swap_%landing';"
    run_query "SELECT 'fee_event' AS event_table, count() FROM default.jupiter_swap_fee_event_landing UNION ALL SELECT 'swap_event' AS event_table, count() FROM default.jupiter_swap_swap_event_landing UNION ALL SELECT 'swaps_event' AS event_table, count() FROM default.jupiter_swap_swaps_event_landing FORMAT PrettyCompact;"
    run_query "SELECT count() AS rows, uniq(signature) AS signatures FROM default.jupiter_swap_route_instruction_landing FORMAT PrettyCompact;"
    ;;
  jupiter-tables)
    run_query "SHOW TABLES FROM default LIKE 'jupiter_swap_%landing';"
    ;;
  jupiter-counts)
    run_query "SELECT count() AS rows, uniq(signature) AS signatures FROM default.jupiter_swap_route_instruction_landing FORMAT PrettyCompact;"
    ;;
  jupiter-route-sample)
    run_query "SELECT instruction_type, in_amount, quoted_out_amount, length(route_plan) AS route_legs FROM default.jupiter_swap_route_instruction_landing LIMIT 3 FORMAT PrettyCompact;"
    ;;
  jupiter-event-sample)
    run_query "SELECT amm, input_amount, output_amount FROM default.jupiter_swap_swap_event_landing LIMIT 5 FORMAT PrettyCompact;"
    ;;
  token)
    run_query "SHOW TABLES FROM default LIKE 'token_program_%account_landing';"
    run_query "SELECT count() AS rows, uniq(pubkey) AS accounts, min(slot) AS min_slot, max(slot) AS max_slot FROM default.token_program_token_account_landing FORMAT PrettyCompact;"
    ;;
  token-tables)
    run_query "SHOW TABLES FROM default LIKE 'token_program_%account_landing';"
    ;;
  token-counts)
    run_query "SELECT count() AS rows, uniq(pubkey) AS accounts, min(slot) AS min_slot, max(slot) AS max_slot FROM default.token_program_token_account_landing FORMAT PrettyCompact;"
    ;;
  token-sample)
    run_query "SELECT left(pubkey,8) AS account, left(mint,8) AS mint, amount, state FROM default.token_program_token_account_landing LIMIT 3 FORMAT PrettyCompact;"
    ;;
  landing-validation)
    run_query "SELECT table_family, count() AS tables, sum(total_rows) AS rows, formatReadableSize(sum(total_bytes)) AS bytes FROM (SELECT name, total_rows, total_bytes, multiIf(endsWith(name, '_account_landing'), 'account rows', endsWith(name, '_instruction_landing'), 'instruction rows', endsWith(name, '_event_landing'), 'CPI event rows', 'other') AS table_family FROM system.tables WHERE database = 'default' AND endsWith(name, '_landing') AND (startsWith(name, 'jupiter_swap_') OR startsWith(name, 'token_program_'))) GROUP BY table_family ORDER BY rows DESC FORMAT PrettyCompact;"
    run_query "SELECT name, total_rows, formatReadableSize(total_bytes) AS bytes FROM system.tables WHERE database = 'default' AND endsWith(name, '_landing') AND (startsWith(name, 'jupiter_swap_') OR startsWith(name, 'token_program_')) ORDER BY total_bytes DESC, total_rows DESC LIMIT 5 FORMAT PrettyCompact;"
    run_query "SELECT program_id, family_name, instruction_type, slot, left(signature, 8) AS signature, instruction_index, stack_height, absolute_path, amount, decimals, source_name, mode FROM default.token_program_transfer_checked_instruction_landing LIMIT 5 FORMAT PrettyCompact;"
    ;;
  *)
    echo "Usage: $0 {jupiter|jupiter-tables|jupiter-counts|jupiter-route-sample|jupiter-event-sample|token|token-tables|token-counts|token-sample|landing-validation|health}" >&2
    exit 2
    ;;
esac
