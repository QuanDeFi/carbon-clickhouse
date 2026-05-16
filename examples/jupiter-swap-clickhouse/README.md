# Jupiter Swap ClickHouse Example

This example runs a bounded real RPC block range through the generated Jupiter
Swap ClickHouse instruction and CPI-event landing tables.

## Required Environment

Create `.env` from `.env.example`:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=<provider-rpc-url>
BLOCK_CRAWLER_START_SLOT=<start-slot>
BLOCK_CRAWLER_END_SLOT=<end-slot>
PROMETHEUS_METRICS_ADDR=0.0.0.0:9464
LOG_LEVEL=info
```

Use the production/provider RPC URL from the local `.env`. The public
mainnet-beta endpoint is not reliable enough for this smoke test.

## Run

```sh
cargo run -p jupiter-swap-clickhouse-carbon-example
```

Or pass the slot range explicitly:

```sh
cargo run -p jupiter-swap-clickhouse-carbon-example -- --start-slot 417942118 --end-slot 417942119
```

The example exposes Carbon metrics for Prometheus at
`PROMETHEUS_METRICS_ADDR` and keeps log metrics enabled. Use
`monitoring/compose.yaml` to run the local Prometheus/Grafana stack.
