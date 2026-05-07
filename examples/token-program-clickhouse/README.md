# Token Program ClickHouse Example

This example runs a real Token Program account snapshot into generated ClickHouse landing tables.

It uses RPC `getProgramAccounts` or Helius gPA v2 with a token-account filter, decodes accounts with `TokenProgramDecoder`, and writes rows through the generated Token Program ClickHouse account processor.

## What This Tests

- Token Program account decoding from real mainnet account data.
- Generated Token Program ClickHouse account table bootstrap.
- Generated typed account rows for the token account family.
- `ClickHouseAccountProcessor` and account metrics.
- Per `(table, partition)` ClickHouse writer buffering and shutdown drain.
- Optional async-wait insert settings when explicitly enabled.

The default path tests token accounts because it can be bounded safely with owner and/or mint filters. The generated decoder also has mint and multisig ClickHouse rows, but unbounded GPA over all Token Program mints is not a safe default.

## Required Environment

Create `.env` from `.env.example`:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=https://api.mainnet-beta.solana.com
TOKEN_ACCOUNT_OWNER=<wallet-pubkey>
# TOKEN_MINT=<mint-pubkey>
LOG_LEVEL=info
PROMETHEUS_METRICS_ADDR=0.0.0.0:9464
```

At least one of these filters must be set:

- `TOKEN_ACCOUNT_OWNER`: token account owner wallet, matched at token account data offset 32.
- `TOKEN_MINT`: token mint, matched at token account data offset 0.

Using both fetches the intersection.

## Run

```sh
cargo run -p token-program-clickhouse-carbon-example
```

The example also exposes Carbon metrics for Prometheus at
`PROMETHEUS_METRICS_ADDR` and keeps log metrics enabled. Use
`monitoring/compose.yaml` to run the local Prometheus/Grafana stack.

To use Helius gPA v2 instead:

```sh
HELIUS_RPC_URL='https://mainnet.helius-rpc.com/?api-key=YOUR_KEY' \
cargo run -p token-program-clickhouse-carbon-example -- --source helius-gpa-v2
```

To opt into ClickHouse async inserts with `wait_for_async_insert=1`:

```sh
CLICKHOUSE_ASYNC_INSERT=true \
CLICKHOUSE_ASYNC_INSERT_BUSY_TIMEOUT_MS=1000 \
cargo run -p token-program-clickhouse-carbon-example
```

## Instruction-Side Real-World Test

`examples/jupiter-swap-clickhouse` is the instruction-side real-world test. It validates real RPC blocks into generated Jupiter instruction and CPI-event ClickHouse rows.
