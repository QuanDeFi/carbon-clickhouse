#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"

echo "-- demo/example processes --"
pgrep -af 'jupiter-swap-clickhouse-carbon-example|token-program-clickhouse-carbon-example|cargo run -p jupiter-swap-clickhouse-carbon-example|cargo run -p token-program-clickhouse-carbon-example' \
  | rg -v 'pgrep|detect-running-demo-processes|bash -c' || true

echo
echo "-- known user services --"
systemctl --user --no-pager --full status \
  carbon-jupiter-swap-clickhouse-live.service \
  carbon-token-program-clickhouse.service \
  carbon-jupiter-clickhouse-backfill.service 2>/dev/null \
  | sed -n '1,120p' || true

echo
echo "-- metrics ports --"
for port in 9464 9465; do
  if port_listener "$port"; then
    :
  else
    echo "$port: not listening"
  fi
done
