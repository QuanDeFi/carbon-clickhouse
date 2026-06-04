Before we start the examples, we verify the local stack from the outside.

Docker Compose shows ClickHouse, Prometheus, and Grafana are up. ClickHouse
answers `SELECT 1`, Prometheus is ready, Grafana health is green, and the
configured RPC endpoint answers `getHealth`.

This is not a data test yet. It simply proves that the database, metrics,
dashboard, and Solana network dependency are reachable before we start writing
anything.
