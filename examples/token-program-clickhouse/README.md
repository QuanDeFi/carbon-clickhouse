# Token Program ClickHouse Example

This example runs a real Token Program account snapshot into generated ClickHouse landing tables.

It uses RPC `getProgramAccounts` or Helius gPA v2 with a token-account filter, decodes accounts with `TokenProgramDecoder`, and writes rows through the generated Token Program ClickHouse account processor.

The default path tests token accounts because it can be bounded safely with owner and/or mint filters. The generated decoder also has mint and multisig ClickHouse rows, but unbounded GPA over all Token Program mints is not a safe default.

## Required Environment

Create `.env` from `.env.example`:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=<provider-rpc-url>
TOKEN_ACCOUNT_OWNER=<wallet-pubkey>
# TOKEN_MINT=<mint-pubkey>
LOG_LEVEL=info
PROMETHEUS_METRICS_ADDR=0.0.0.0:9464
```

Use the production/provider RPC URL from the local `.env`. The public
mainnet-beta endpoint is not reliable enough for bounded account smoke tests.

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

## ClickHouse Tables

The example bootstraps one typed account landing table per generated Token
Program account family:

<table>
  <colgroup>
    <col width="34%" />
    <col width="66%" />
  </colgroup>
  <thead>
    <tr>
      <th>Table</th>
      <th>What it stores</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>token_program_<wbr>mint_account_<wbr>landing</code></td>
      <td>SPL Token mint account snapshots, including mint configuration and supply state. This table exists for mint-family account data and can be empty when the example is run with only token-account filters.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>multisig_account_<wbr>landing</code></td>
      <td>SPL Token multisig account snapshots, including threshold and signer state. This table exists for multisig-family account data and can be empty when the example is run with only token-account filters.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>token_account_<wbr>landing</code></td>
      <td>SPL Token holding-account snapshots matching the configured owner and/or mint filters. This is the default bounded real-world path for validating account-family ClickHouse ingestion.</td>
    </tr>
  </tbody>
</table>

`account_id` is a deterministic landing identifier based on program, pubkey,
slot, and account type. Landing rows are append-only snapshots. If the same
token account is fetched again at a later slot, the table will contain another
row for the same `pubkey` with a different `slot` and `account_id`.

`transaction_signature` is expected to be `NULL` for account snapshots sourced
from GPA because those snapshots are not tied to a single transaction. It is
only populated when the datasource provides account updates with transaction
context.

## RPC Source Notes

The default `--source rpc` path calls standard Solana JSON-RPC
`getProgramAccounts` via `RPC_URL`. Free/trial RPC tiers vary by provider and
method. Some providers rate-limit or disable bounded `getProgramAccounts`; this
is an RPC-provider capability issue, not a ClickHouse sink issue.

The `--source helius-gpa-v2` path calls Helius `getProgramAccountsV2` via
`HELIUS_RPC_URL` and supports pagination through `--helius-page-limit` and
`--helius-changed-since-slot`.
