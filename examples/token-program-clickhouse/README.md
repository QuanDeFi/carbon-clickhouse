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

| Table | Payload columns | When rows are produced |
| --- | --- | --- |
| `token_program_mint_account_landing` | `mint_authority`, `supply`, `decimals`, `is_initialized`, `freeze_authority` | When the datasource emits Token Program mint accounts. The default token-account filter does not produce mint rows. |
| `token_program_multisig_account_landing` | `m`, `n`, `is_initialized`, `signers` | When the datasource emits Token Program multisig accounts. The default token-account filter does not produce multisig rows. |
| `token_program_token_account_landing` | `mint`, `token_owner`, `amount`, `delegate`, `state`, `is_native`, `delegated_amount`, `close_authority` | The default real-world path. Rows are produced for token accounts matching `TOKEN_ACCOUNT_OWNER`, `TOKEN_MINT`, or both. |

All account tables share these metadata columns:

```text
program_id, family_name, account_type, account_id, slot, pubkey,
transaction_signature, lamports, owner, executable, rent_epoch, source_name,
mode, decoder_version, ingest_ts, partition_slot
```

`account_id` is a deterministic landing identifier based on program, pubkey,
slot, and account type. Landing rows are append-only snapshots. If the same
token account is fetched again at a later slot, the table will contain another
row for the same `pubkey` with a different `slot` and `account_id`.

`transaction_signature` is expected to be `NULL` for account snapshots sourced
from GPA because those snapshots are not tied to a single transaction. It is
only populated when the datasource provides account updates with transaction
context.

## Postgres Parity

The generated ClickHouse output covers every Token Program account family that
the generated Postgres account module covers:

| ClickHouse table | What it stores | Postgres equivalent |
| --- | --- | --- |
| `token_program_mint_account_landing` | SPL Token mint account snapshots: authority, supply, decimals, initialization state, and freeze authority. | `mint_account` |
| `token_program_multisig_account_landing` | SPL Token multisig account snapshots: signature threshold, signer count, initialization state, and signer pubkeys. | `multisig_account` |
| `token_program_token_account_landing` | SPL Token account snapshots: mint, token owner, raw amount, delegate/native/close-authority state, and delegated amount. | `token_account` |

The decoded account payload fields are complete relative to Postgres. The main
differences are table shape and semantics:

- Postgres tables use compact generated columns such as `__pubkey`, `__slot`,
  and payload fields.
- ClickHouse landing tables add deterministic `account_id`, source/mode/version
  metadata, ingest time, partition key, lamports, Solana account owner,
  executable flag, rent epoch, and optional transaction signature.
- The Token account payload field named `owner` in Postgres is named
  `token_owner` in ClickHouse to avoid ambiguity with the Solana account
  metadata field `owner`.
- Postgres account rows are generated for upsert-style storage keyed by account
  pubkey. ClickHouse account landing rows are append-only snapshots keyed by
  deterministic landing identity.

## Inspecting Token Account Rows

Basic row counts:

```sql
SELECT
  count() AS rows,
  uniqExact(account_id) AS account_ids,
  uniqExact(pubkey) AS pubkeys,
  min(slot) AS min_slot,
  max(slot) AS max_slot
FROM token_program_token_account_landing;
```

Latest snapshot per token account:

```sql
SELECT
  pubkey,
  argMax(mint, slot) AS mint,
  argMax(token_owner, slot) AS token_owner,
  argMax(amount, slot) AS amount,
  max(slot) AS latest_slot
FROM token_program_token_account_landing
GROUP BY pubkey
ORDER BY latest_slot DESC
LIMIT 50;
```

Token balances by mint for the latest rows in the table:

```sql
SELECT
  mint,
  sum(amount) AS raw_amount
FROM
(
  SELECT
    pubkey,
    argMax(mint, slot) AS mint,
    argMax(amount, slot) AS amount
  FROM token_program_token_account_landing
  GROUP BY pubkey
)
GROUP BY mint
ORDER BY raw_amount DESC
LIMIT 50;
```

The `amount` column is the raw SPL Token amount. Join it with mint `decimals`
from `token_program_mint_account_landing` or another token metadata source if
you need UI-denominated balances.

## RPC Source Notes

The default `--source rpc` path calls standard Solana JSON-RPC
`getProgramAccounts` via `RPC_URL`. Free/trial RPC tiers vary by provider and
method. Some providers rate-limit or disable bounded `getProgramAccounts`; this
is an RPC-provider capability issue, not a ClickHouse sink issue.

The `--source helius-gpa-v2` path calls Helius `getProgramAccountsV2` via
`HELIUS_RPC_URL` and supports pagination through `--helius-page-limit` and
`--helius-changed-since-slot`.

## Instruction-Side Real-World Test

`examples/jupiter-swap-clickhouse` is the instruction-side real-world test. It validates real RPC blocks into generated Jupiter instruction and CPI-event ClickHouse rows.
