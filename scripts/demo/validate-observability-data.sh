#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env
ensure_demo_dirs

OUT_DIR="${1:-$ROOT/demo-artifacts/review-human/logs}"
mkdir -p "$OUT_DIR"
QUERY_FILE="$OUT_DIR/prometheus-query.txt"
SUMMARY_FILE="$OUT_DIR/prometheus-query-summary.json"

queries=(
  "increase(carbon_updates_processed_total[2h])"
  "increase(carbon_updates_processed_total[30m])"
  "last_over_time(clickhouse_instructions_inserted[2h])"
  "last_over_time(clickhouse_accounts_inserted[2h])"
  "last_over_time(carbon_updates_processed_total[2h])"
  "increase(carbon_updates_failed_total[30m])"
  "last_over_time(clickhouse_instructions_buffered_rows[2h])"
  "last_over_time(clickhouse_accounts_buffered_rows[2h])"
  "increase(clickhouse_instructions_flush_failed_batches[30m])"
  "increase(clickhouse_accounts_flush_failed_batches[30m])"
  "up"
)

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

for query in "${queries[@]}"; do
  if ! curl -fsS --get --data-urlencode "query=$query" "http://localhost:9090/api/v1/query" > "$tmp"; then
    continue
  fi
  count="$(jq '.data.result | length' "$tmp")"
  positive_count="$(jq '[.data.result[].value[1] | tonumber? // 0 | select(. > 0)] | length' "$tmp")"
  if [[ "$count" -gt 0 && "$positive_count" -gt 0 ]]; then
    printf '%s\n' "$query" > "$QUERY_FILE"
    jq --arg query "$query" '{query: $query, result_count: (.data.result | length), first_result: .data.result[0]}' "$tmp" > "$SUMMARY_FILE"
    echo "prometheus_query=$query"
    echo "result_count=$count"
    exit 0
  fi
done

echo "No useful Prometheus query returned data" >&2
exit 1
