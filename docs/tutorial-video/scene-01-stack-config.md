Before starting, we verify the local stack.

Docker Compose shows ClickHouse, Prometheus, and Grafana are up. ClickHouse
answers `SELECT 1`, Prometheus is ready, Grafana health is green, and the
configured RPC endpoint answers `getHealth`.

That proves local services and Solana network access are reachable.
