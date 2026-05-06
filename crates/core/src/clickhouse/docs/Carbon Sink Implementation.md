## Carbon ClickHouse Sink

The ClickHouse sink is a generator-backed landing-table backend that fits Carbon's existing processor architecture.

The implementation is landing-only. It is not a warehouse serving layer, replay convergence system, or online deduplication system.

## Implementation Scope

The ClickHouse path includes:

1. A generic ClickHouse runtime inside `carbon-core`.
2. Processor finalization so buffered processors drain on shutdown.
3. Typed instruction and CPI-event ClickHouse coverage for `jupiter-swap-decoder`.
4. Typed account ClickHouse coverage for `token-program-decoder`.
5. Generic core landing rows/processors for transaction, account-deletion, and block-details surfaces.
6. Generated Jupiter transaction aggregation rows.
7. Renderer support for generated typed account, instruction, CPI-event, and transaction landing rows.
8. A per-buffer ClickHouse writer for independent Carbon processes writing into the same table families.
9. Explicit synchronous and async-wait insert settings, with synchronous inserts as the default.

## How It Fits Into Carbon

ClickHouse uses normal Carbon processors, not a separate pipeline type.

The runtime shape is:

1. One or more datasources emit Carbon updates.
2. Carbon builds decoded account, instruction, transaction, or block inputs.
3. A ClickHouse-backed processor receives decoded inputs.
4. Decoder-owned code maps those inputs into typed ClickHouse row families.
5. The generic ClickHouse batch writer groups rows by table and partition.
6. Batches are inserted into ClickHouse over HTTP with `JSONEachRow`.

That keeps the ClickHouse path aligned with Carbon's existing model:

- Pipelines route data.
- Processors own side effects.
- Decoder crates own schema and storage-specific row mapping.

## Processor Finalization

Buffered ClickHouse writes require processor finalization. A processor can process rows that remain in memory until a batch threshold, timer, explicit flush, or shutdown drain fires.

The core lifecycle includes:

- `Processor::finalize()`
- pipe wrappers forwarding `finalize()`
- `Pipeline` calling `finalize_pipes()` on shutdown paths before exporter shutdown

ClickHouse processors call writer shutdown from `finalize()`, which drains all remaining buffers and stops the background worker. This preserves processed rows during datasource cancellation, Ctrl-C shutdown, and channel closure.

## What Lives In `carbon-core`

The generic ClickHouse runtime in `carbon-core` is split by responsibility:

- `config.rs` - endpoint, database, auth, source metadata, batching, and insert settings
- `http.rs` - minimal HTTP transport over `reqwest`
- `admin.rs` - explicit schema/bootstrap execution
- `rows/mod.rs` - row/table traits, row context, multi-row contract, and deterministic ID helpers
- `writer.rs` - per-buffer batch writer and background flush worker
- `surface_rows.rs` - generic transaction, account-deletion, and block-details landing rows
- `processors.rs` - ClickHouse instruction, account, transaction, account-deletion, and block-details processors
- `metrics.rs` - shared ClickHouse metrics for all ClickHouse processor families

Important boundary:

- Core knows how to talk to ClickHouse.
- Core knows how to buffer, flush, retry in memory, and shut down rows.
- Core does not know decoder-specific schema details.

## Write Path And Insert Model

The sink uses client-side batching and HTTP `JSONEachRow` inserts.

Writer behavior:

- Rows are buffered in memory.
- Buffers are keyed by `(table, partition_key())`.
- Size-based flush happens per buffer, not globally across all buffered rows.
- A background worker starts lazily on the first async writer operation.
- The background timer flushes only stale buffers whose own `last_flush` exceeds `flush_interval`.
- `flush()` drains all buffers for explicit/manual drains.
- `shutdown()` drains all buffers and stops the background worker.
- Processors call `shutdown()` from `finalize()`.
- Failed batches are reinserted into the same buffer and surfaced on the next writer operation or shutdown.
- Rows are serialized with `serde::Serialize`.
- Inserts go through ClickHouse HTTP, not the native protocol.

This matters for typed landing tables because one decoder wrapper can emit rows for many tables. A hot table does not force unrelated cold buffers to flush early, and a cold table still flushes when idle.

## Insert Settings

Synchronous inserts are the default:

- `ClickHouseConfig::new(...)` defaults to `ClickHouseInsertSettings::Sync`.
- `ClickHouseConfig::from_database_url(...)` defaults to `ClickHouseInsertSettings::Sync`.
- Generated setup helpers remain sync by default.

Production live ingestion can opt into server-side async inserts:

```rust
use carbon_core::clickhouse::{
    ClickHouseAsyncInsertSettings,
    ClickHouseInsertSettings,
};

let config = config.with_insert_settings(ClickHouseInsertSettings::AsyncWait(
    ClickHouseAsyncInsertSettings {
        busy_timeout_ms: Some(1_000),
        max_data_size: None,
        max_query_number: None,
        deduplicate: Some(true),
    },
));
```

Async mode always sends:

- `async_insert=1`
- `wait_for_async_insert=1`

The public sink API does not expose `wait_for_async_insert=0`. Fire-and-forget inserts hide server-side insert failures from the Carbon process, which does not match this sink's delivery model.

All insert requests include `date_time_input_format=best_effort`; async settings are merged into the same query-setting path.

## Generated Row Scope

The generated row scope is canary-limited and covers account, instruction, CPI-event, and transaction row families:

- Jupiter swap typed instruction landing rows.
- Jupiter swap typed CPI-event landing rows.
- Jupiter swap typed transaction aggregation landing rows.
- Token Program typed account landing rows for mint, token, and multisig.
- Renderer templates for typed account, instruction, CPI-event, and transaction ClickHouse generation.
- `withClickHouse` feature-gated generated modules.

The table model is one typed landing table per generated row family. This avoids the schema, compression, and query-shape problems of one large generic JSON landing table.

Generated row mapping derives structured ClickHouse types from the decoder schema:

- primitives map to native ClickHouse scalar types
- `u128` and `i128` use generated serializer wrappers while retaining `UInt128` and `Int128` columns
- arrays and fixed arrays map to `Array(...)`
- structs and tuples map to generated Rust helper structs and ClickHouse `Tuple(...)`
- fieldless enums map to `Enum8` or `Enum16`
- payload enums map to generated tagged-union helper structs with a variant enum and typed payload fields

Known decoder-owned enum payloads are not stringified. JSON remains only as an explicit fallback for unsupported dynamic shapes.

## Jupiter Example

`examples/jupiter-swap-clickhouse` is the real-world smoke test for the Jupiter instruction, CPI-event, and transaction aggregation path.

It validates:

- `RpcBlockCrawler` datasource
- production RPC URL loading through `.env`
- ClickHouse bootstrap through generated Jupiter migration helpers
- Jupiter instruction and CPI-event decoding
- typed ClickHouse row dispatch
- structured CPI-event payloads such as `route_plan.swap`
- generated transaction aggregation rows in `jupiter_swap_transaction_landing`
- core writer batching and shutdown drain

It does not validate:

- Token Program account rows
- multi-decoder fan-in
- replicated/distributed ClickHouse DDL

Use a bounded slot range when testing the example against production RPC.

## Token Program Example

`examples/token-program-clickhouse` is the real-world smoke test for the account-family path.

It validates:

- filtered RPC `getProgramAccounts` or Helius gPA v2
- generated Token Program ClickHouse account table bootstrap
- Token Program account decoding from real mainnet account data
- generated token account landing rows
- `ClickHouseAccountProcessor`
- account-family metrics
- per-buffer writer flushing and shutdown drain

The example requires a token-account owner and/or mint filter so it does not accidentally fetch the entire Token Program account set. By default it focuses on token accounts because those can be bounded safely. The generated mint and multisig account rows are covered by compile/tests and schema generation, but they are not the default real-world query path.

## Identity, Replay, And Table Semantics

The sink is landing-only and append-only.

Generated rows use deterministic IDs:

- instruction rows use `deterministic_instruction_id(...)`
- CPI-event rows use `deterministic_event_id(...)`
- account rows use `deterministic_account_id(...)`
- transaction rows use `deterministic_transaction_id(...)`
- account-deletion rows use `deterministic_account_deletion_id(...)`
- block-details rows use `deterministic_block_details_id(...)`

That gives stable row identity across reprocessing, which is required for canonicalization, deduplication, or stronger retry idempotency above the landing layer.

ClickHouse primary keys do not enforce uniqueness, so deterministic IDs are identity metadata, not online deduplication by themselves.

The sink does not do:

- online row-level deduplication
- serving-table resolution
- replay convergence
- coverage/range tracking
- durable retry state

The contract is stable landing identity, not full canonical warehouse semantics.

## Metrics And Observability

The sink has its own metrics because batch sinks need different visibility than row-at-a-time sinks.

Tracked metrics:

- inserted rows
- failed rows in failed batches
- buffered row count
- successful flush batches
- failed flush batches
- flush duration histogram

Metrics are separated by processor family:

- `clickhouse.instructions.*`
- `clickhouse.accounts.*`
- `clickhouse.transactions.*`
- `clickhouse.account_deletions.*`
- `clickhouse.block_details.*`

Background flushes and shutdown-triggered flushes record through the shared ClickHouse metrics module, not through processor-local foreground accounting.

## Generator State

The renderer supports:

- decoder manifest support for a `clickhouse` feature
- typed account landing row templates
- typed instruction landing row templates
- typed per-event CPI landing row templates
- typed transaction aggregation row templates
- generated account `clickhouse/mod.rs`
- generated instruction `clickhouse/mod.rs`
- generated transaction `clickhouse/mod.rs`
- generated Cargo dependencies for ClickHouse, `serde`, and `chrono`

The repository keeps ClickHouse decoder output canary-limited; it has not regenerated every decoder with ClickHouse output.

## Validation Commands

These commands cover the ClickHouse runtime, renderer, decoder canaries, and examples:

```bash
pnpm --filter @sevenlabs-hq/carbon-codama-renderer test
pnpm --filter @sevenlabs-hq/carbon-codama-renderer type-check
cargo test -p carbon-core --features clickhouse clickhouse --lib
cargo test -p carbon-jupiter-swap-decoder --features clickhouse clickhouse --lib
cargo check -p carbon-jupiter-swap-decoder --features clickhouse
cargo check -p carbon-token-program-decoder --features clickhouse
cargo check -p jupiter-swap-clickhouse-carbon-example
cargo check -p token-program-clickhouse-carbon-example
git diff --check
cargo fmt --all
```

## Production Model

The scaled deployment model is multiple Carbon processes running in parallel, each with its own datasource/decoder/processor pipeline and each writing to ClickHouse.

For that model:

- Each process owns local buffering and flush state.
- The writer flushes independently per `(table, partition)` buffer.
- Synchronous inserts are the default for backfills and deterministic ingestion.
- Async-wait inserts are available for live production ingestion with many writers.
- ClickHouse table design is one typed landing table per generated row family.

Implementation boundaries:

- Renderer-controlled replicated/distributed DDL modes are not generated.
- Explicit insert deduplication tokens are not exposed.
- Serving/canonicalization tables are not part of the landing sink.
- Coverage/range tracking is not part of the landing sink.
