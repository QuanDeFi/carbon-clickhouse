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

## ClickHouse Tables

The example bootstraps one typed landing table per generated Jupiter Swap
instruction family plus one typed CPI/event landing table. Every table stores
common landing metadata plus the typed payload fields owned by the decoded
instruction or event families. Empty instruction tables are expected when the
selected slot range does not contain that instruction type.

A few Jupiter instruction structs are payload-free. Those rows are not missing
data; the decoded instruction has no instruction-data fields, so the row records
the occurrence, identity, slot/signature/path metadata, and sink metadata.

Instruction landing tables:

| Table | What it stores | Payload fields | Postgres equivalent |
| --- | --- | --- | --- |
| `jupiter_swap_claim_instruction_landing` | Claim instruction rows. | `id` | `claim_instruction` |
| `jupiter_swap_claim_token_instruction_landing` | Claim-token instruction rows. | `id` | `claim_token_instruction` |
| `jupiter_swap_close_token_instruction_landing` | Close-token instruction rows. | `id`, `burn_all` | `close_token_instruction` |
| `jupiter_swap_close_wsol_token_account_instruction_landing` | Close wrapped-SOL token account instruction rows. The decoded instruction payload is empty. | Metadata only | `close_wsol_token_account_instruction` |
| `jupiter_swap_create_token_account_instruction_landing` | Create Jupiter token account instruction rows. | `bump` | `create_token_account_instruction` |
| `jupiter_swap_create_token_ledger_instruction_landing` | Create token ledger instruction rows. The decoded instruction payload is empty. | Metadata only | `create_token_ledger_instruction` |
| `jupiter_swap_exact_out_route_instruction_landing` | Exact-out route instruction rows. | `route_plan`, `out_amount`, `quoted_in_amount`, `slippage_bps`, `platform_fee_bps` | `exact_out_route_instruction` |
| `jupiter_swap_exact_out_route_v2_instruction_landing` | Exact-out route V2 instruction rows. | `out_amount`, `quoted_in_amount`, `slippage_bps`, `platform_fee_bps`, `positive_slippage_bps`, `route_plan` | `exact_out_route_v2_instruction` |
| `jupiter_swap_route_instruction_landing` | Standard route instruction rows. | `route_plan`, `in_amount`, `quoted_out_amount`, `slippage_bps`, `platform_fee_bps` | `route_instruction` |
| `jupiter_swap_route_v2_instruction_landing` | Standard route V2 instruction rows. | `in_amount`, `quoted_out_amount`, `slippage_bps`, `platform_fee_bps`, `positive_slippage_bps`, `route_plan` | `route_v2_instruction` |
| `jupiter_swap_route_with_token_ledger_instruction_landing` | Route instruction rows that source the input amount from token ledger state. | `route_plan`, `quoted_out_amount`, `slippage_bps`, `platform_fee_bps` | `route_with_token_ledger_instruction` |
| `jupiter_swap_set_token_ledger_instruction_landing` | Set token ledger instruction rows. The decoded instruction payload is empty. | Metadata only | `set_token_ledger_instruction` |
| `jupiter_swap_shared_accounts_exact_out_route_instruction_landing` | Shared-accounts exact-out route instruction rows. | `id`, `route_plan`, `out_amount`, `quoted_in_amount`, `slippage_bps`, `platform_fee_bps` | `shared_accounts_exact_out_route_instruction` |
| `jupiter_swap_shared_accounts_exact_out_route_v2_instruction_landing` | Shared-accounts exact-out route V2 instruction rows. | `id`, `out_amount`, `quoted_in_amount`, `slippage_bps`, `platform_fee_bps`, `positive_slippage_bps`, `route_plan` | `shared_accounts_exact_out_route_v2_instruction` |
| `jupiter_swap_shared_accounts_route_instruction_landing` | Shared-accounts route instruction rows. | `id`, `route_plan`, `in_amount`, `quoted_out_amount`, `slippage_bps`, `platform_fee_bps` | `shared_accounts_route_instruction` |
| `jupiter_swap_shared_accounts_route_v2_instruction_landing` | Shared-accounts route V2 instruction rows. | `id`, `in_amount`, `quoted_out_amount`, `slippage_bps`, `platform_fee_bps`, `positive_slippage_bps`, `route_plan` | `shared_accounts_route_v2_instruction` |
| `jupiter_swap_shared_accounts_route_with_token_ledger_instruction_landing` | Shared-accounts route rows that source the input amount from token ledger state. | `id`, `route_plan`, `quoted_out_amount`, `slippage_bps`, `platform_fee_bps` | `shared_accounts_route_with_token_ledger_instruction` |

CPI/event landing tables:

| Table | What it stores | Payload fields | Postgres equivalent |
| --- | --- | --- | --- |
| `jupiter_swap_cpi_event_landing` | All generated Jupiter CPI/event rows, with `event_type` as the discriminator. | Event-prefixed typed columns for fee, single swap, batched swaps, candidate results, quote errors, and best-swap output violations. Array/composite event payloads use `*_present` markers plus non-null structured arrays. | `cpi_events` |

Instruction tables share these metadata columns:

```text
program_id, family_name, instruction_type, instruction_id, slot, signature,
instruction_index, stack_height, absolute_path, source_name, mode,
decoder_version, ingest_ts, chain_time, partition_time, block_hash, tx_index
```

The CPI/event table uses the same transaction/instruction metadata and adds
`event_type`, `event_id`, and `event_seq`. `instruction_id` and `event_id` are
deterministic landing identifiers. Landing tables are append-only; repeated
backfills can produce multiple rows for the same logical instruction/event if a
slot range is replayed.

`event_type = 'swap_event'` rows store a single swap event in the
`swap_event_*` columns. `event_type = 'swaps_event'` rows store an event that
contains an array of swap events in `swaps_event_swap_events`.

## Postgres Parity

The generated ClickHouse output mirrors the generated Postgres table count for
Jupiter Swap: 17 instruction tables plus one CPI/event table.

CPI/event coverage is complete and now lands in one table, like Postgres.
Postgres uses `cpi_events.name` as the discriminator and `data JSONB` as the
payload. ClickHouse uses `jupiter_swap_cpi_event_landing.event_type` as the
discriminator and stores each known event payload in native typed columns.

The current ClickHouse rows are not exact column-for-column copies of Postgres:

- Postgres instruction/event tables include raw `__accounts` JSON.
- ClickHouse instruction/event landing tables currently store
  `absolute_path`, transaction metadata, deterministic landing IDs, sink
  metadata, and typed payload columns, but not raw account-meta JSON.
- Postgres `route_plan` is `JSONB`; ClickHouse `route_plan` is
  `Array(Tuple(...))`.
- Postgres CPI/event payloads are `data JSONB`; ClickHouse CPI/event payloads
  are event-prefixed typed columns in `jupiter_swap_cpi_event_landing`.
- Postgres rows support generated upsert/delete/lookup operations; ClickHouse
  landing tables are append-only.

There is no generated ClickHouse transaction table in this example. That matches
the current Postgres-aligned generated decoder surface. Any old
`jupiter_swap_transaction_landing` table is stale local state from an older
transaction-aggregation experiment and is not produced by current code.

## Inspecting `route_plan`

`route_plan` is a structured ClickHouse value:

```text
Array(Tuple(swap Tuple(...), percent/bps, input_index, output_index))
```

Query it as JSON when browsing rows:

```sql
SELECT
  signature,
  instruction_index,
  toJSONString(route_plan) AS route_plan_json
FROM jupiter_swap_route_instruction_landing
LIMIT 10;
```

For analysis, extract only the fields you need:

```sql
SELECT
  signature,
  arrayMap(
    x -> tupleElement(tupleElement(x, 'swap'), 'variant'),
    route_plan
  ) AS swap_variants,
  arrayMap(x -> tupleElement(x, 'percent'), route_plan) AS percents,
  arrayMap(x -> tupleElement(x, 'input_index'), route_plan) AS input_indexes,
  arrayMap(x -> tupleElement(x, 'output_index'), route_plan) AS output_indexes
FROM jupiter_swap_route_instruction_landing
LIMIT 10;
```

For V2 route tables, use `bps` instead of `percent`:

```sql
SELECT
  signature,
  arrayMap(
    x -> tupleElement(tupleElement(x, 'swap'), 'variant'),
    route_plan
  ) AS swap_variants,
  arrayMap(x -> tupleElement(x, 'bps'), route_plan) AS bps
FROM jupiter_swap_route_v2_instruction_landing
LIMIT 10;
```
