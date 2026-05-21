# Token Program ClickHouse Example

This example runs fixed real USDC Token Program account snapshots into generated ClickHouse landing tables.

It fetches four hardwired mainnet USDC accounts with `getMultipleAccounts`:
the USDC mint, the USDC mint authority multisig, the USDC freeze authority
multisig, and one USDC token holding account. All accounts are decoded with
`TokenProgramDecoder` and written through the generated Token Program
ClickHouse account processor.

The default path populates all three generated Token Program account landing
tables without scanning Token Program accounts or paginating account indexes.

## Required Environment

Create `.env` from `.env.example`:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=<provider-rpc-url>
PROMETHEUS_METRICS_ADDR=0.0.0.0:9465
LOG_LEVEL=info
```

Use the production/provider RPC URL from the local `.env`. The public
mainnet-beta endpoint is not reliable enough for account smoke tests. The USDC
mint, USDC authority multisigs, and USDC token account are hardwired in the
example.

## Run

```sh
cargo run -p token-program-clickhouse-carbon-example
```

The example uses `getMultipleAccounts` through `RPC_URL`.

The example exposes Carbon/ClickHouse metrics at `PROMETHEUS_METRICS_ADDR` and
keeps the process alive briefly after the snapshot so Prometheus can scrape the
final counters.

To opt into ClickHouse async inserts with `wait_for_async_insert=1`:

```sh
CLICKHOUSE_ASYNC_INSERT=true cargo run -p token-program-clickhouse-carbon-example
```

Generated ClickHouse table schema is checked before ingestion. The sink creates
missing generated tables, adds missing generated columns, and repairs only
proven enum-extension drift. Other schema drift fails before ingestion.
Destructive drop/recreate cleanup is intentionally not exposed through this
example.

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
      <td>The USDC mint account snapshot. This records the token's supply state, decimals, and the authorities that can mint new USDC or freeze USDC token accounts.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>multisig_account_<wbr>landing</code></td>
      <td>The SPL Token multisig accounts currently referenced by the USDC mint as mint authority and freeze authority. These rows show the signer threshold and signer set controlling those USDC authority paths.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>token_account_<wbr>landing</code></td>
      <td>One fixed USDC token holding-account snapshot. This validates the token-account family with a small real holder/account sample.</td>
    </tr>
  </tbody>
</table>

`account_id` is a deterministic landing identifier based on program, pubkey,
slot, and account type. Landing rows are append-only snapshots. If the same
token account is fetched again at a later slot, the table will contain another
row for the same `pubkey` with a different `slot` and `account_id`.

`transaction_signature` is expected to be `NULL` because these snapshots are
current account reads, not transaction-scoped account updates.
