#!/usr/bin/env bash
# Flush prior-run Prometheus series so each demo recording starts with fresh
# observability data. carbon-prometheus is dedicated to this tutorial (its only
# scrape job is "carbon-local"), so flushing its series does not affect any
# unrelated monitoring on the host.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env
ensure_demo_dirs
RESET_LOG="$ROOT/demo-artifacts/logs/prometheus-reset.log"
exec > >(tee "$RESET_LOG") 2>&1

PROM_URL="${PROMETHEUS_URL:-http://localhost:9090}"
COMPOSE="$ROOT/monitoring/compose.yaml"
# {job="carbon-local"} url-encoded
MATCH='%7Bjob%3D%22carbon-local%22%7D'

if [[ "${DEMO_ALLOW_CLICKHOUSE_RESET:-}" != "true" ]]; then
  log_warn "Prometheus reset skipped (set DEMO_ALLOW_CLICKHOUSE_RESET=true to flush series)."
  exit 0
fi

# The admin API (needed for delete_series) is enabled in monitoring/compose.yaml.
# Probe it; if it is not active yet, recreate the container to pick up the flag.
admin_api_ready() {
  local code
  code="$(curl -s -o /dev/null -w '%{http_code}' -X POST "$PROM_URL/api/v1/admin/tsdb/clean_tombstones" || echo 000)"
  [[ "$code" == "204" ]]
}

if ! admin_api_ready; then
  log_info "Enabling Prometheus admin API (recreating carbon-prometheus)"
  docker compose -f "$COMPOSE" up -d prometheus >/dev/null 2>&1 || true
  for _ in $(seq 1 30); do
    curl -fsS "$PROM_URL/-/ready" >/dev/null 2>&1 && break
    sleep 1
  done
fi

if ! admin_api_ready; then
  log_warn "Prometheus admin API unavailable; skipping series flush (Grafana may show prior-run metrics)."
  exit 0
fi

log_info "Flushing prior-run Prometheus series (job=carbon-local)"
if curl -fsS -X POST "$PROM_URL/api/v1/admin/tsdb/delete_series?match[]=$MATCH" >/dev/null 2>&1; then
  curl -fsS -X POST "$PROM_URL/api/v1/admin/tsdb/clean_tombstones" >/dev/null 2>&1 || true
  log_info "Prometheus series flushed; fresh metrics will be scraped during this run."
else
  log_warn "Prometheus delete_series request failed; continuing."
fi
