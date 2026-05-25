#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=scripts/demo/common.sh
source "$ROOT/scripts/demo/common.sh"
load_demo_env
cd "$ROOT"

docker compose -f monitoring/compose.yaml up -d
docker compose -f monitoring/compose.yaml ps
curl -fsS 'http://carbon:carbon@localhost:8123/?query=SELECT%201' | mask_secrets
curl -fsS 'http://localhost:9090/-/ready' | mask_secrets
curl -fsS 'http://localhost:3000/api/health' | mask_secrets
