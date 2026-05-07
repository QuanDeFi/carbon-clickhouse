# Carbon Observability Stack

This stack runs Prometheus and Grafana for local Carbon ClickHouse sink monitoring.

## Containers

- Prometheus: `prom/prometheus:v3.11.3`
- Grafana: `grafana/grafana:13.0.1`

## Start

```sh
docker compose -f monitoring/compose.yaml up -d
```

Open:

- Prometheus: http://localhost:9090
- Grafana: http://localhost:3000

Default Grafana login:

- User: `admin`
- Password: `admin`

## Carbon Metrics Endpoint

The ClickHouse examples expose Carbon metrics at:

```text
0.0.0.0:9464/metrics
```

Override the listen address when running multiple Carbon processes on one host:

```sh
PROMETHEUS_METRICS_ADDR=0.0.0.0:9465 cargo run -p jupiter-swap-clickhouse-carbon-example
```

Prometheus currently uses host networking and scrapes the local Carbon process at:

```text
localhost:9464
```

For multiple local Carbon processes, add more targets to
`monitoring/prometheus/prometheus.yml` with stable labels such as
`pipeline_id`, `decoder`, `datasource`, and `mode`.

## Recommended Labels

Use Prometheus scrape labels for process identity:

```yaml
labels:
  pipeline_id: jupiter-live-01
  decoder: jupiter_swap
  datasource: rpc_block_crawler
  mode: live
```

Keep high-cardinality values out of labels. Do not label by signature, pubkey,
slot, or ClickHouse partition key.

## Useful Queries

Carbon process health:

```promql
carbon_updates_queued
rate(carbon_updates_processed_total[1m])
rate(carbon_updates_failed_total[1m])
```

ClickHouse sink health:

```promql
clickhouse_instructions_buffered_rows
clickhouse_instructions_buffered_bytes
clickhouse_instructions_active_buffers
rate(clickhouse_instructions_retries[1m])
rate(clickhouse_instructions_flush_failed_batches[1m])
rate(clickhouse_instructions_backpressure_rejected[1m])
```

Account sink health:

```promql
clickhouse_accounts_buffered_rows
clickhouse_accounts_buffered_bytes
clickhouse_accounts_active_buffers
rate(clickhouse_accounts_retries[1m])
rate(clickhouse_accounts_flush_failed_batches[1m])
rate(clickhouse_accounts_backpressure_rejected[1m])
```

## Notes

The stack monitors Carbon process metrics. ClickHouse server-level monitoring
should be added separately through ClickHouse system tables or a ClickHouse
exporter when production topology is known.
