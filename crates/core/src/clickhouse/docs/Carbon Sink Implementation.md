## Carbon ClickHouse Sink

`clickhouse-upstream-v1` adds a generator-backed ClickHouse landing-table sink that fits Carbon's existing processor architecture.

The implemented scope is intentionally landing-only. It is not a full warehouse layer, serving layer, or replay convergence system.

## What this branch adds

This branch adds:

1. A generic ClickHouse runtime inside `carbon-core`.
2. A processor finalization hook so buffered processors can drain on shutdown.
3. Typed instruction and CPI-event ClickHouse canary coverage for `jupiter-swap-decoder`.
4. Typed account ClickHouse canary coverage for `token-program-decoder`.
5. Renderer support for generated typed account, instruction, and CPI-event landing rows.
6. A per-buffer ClickHouse writer suitable for many independent Carbon processes writing into the same table family.
7. Explicit synchronous and async-wait insert settings, with synchronous inserts as the default.

## How it fits into Carbon

ClickHouse is implemented as normal Carbon processors, not as a new pipeline type.

The runtime shape is:

1. One or more datasources emit Carbon updates.
2. Carbon builds decoded account, instruction, transaction, or block inputs.
3. A ClickHouse-backed processor receives decoded inputs.
4. Decoder-owned code maps those inputs into typed ClickHouse row families.
5. The generic ClickHouse batch writer groups rows by table and partition.
6. Batches are inserted into ClickHouse over HTTP with `JSONEachRow`.

That choice keeps the ClickHouse path aligned with Carbon's existing model:

- Pipelines route data.
- Processors own side effects.
- Decoder crates own schema and storage-specific row mapping.

## The core architectural change: processor finalization

The most important Carbon-core change is processor finalization.

Because ClickHouse writes are buffered, accepted rows may still be in memory when the pipeline shuts down. This branch adds:

- `Processor::finalize()`
- pipe wrappers forwarding `finalize()`
- `Pipeline` calling `finalize_pipes()` on shutdown paths before exporter shutdown

That lets ClickHouse processors flush buffered rows deterministically on datasource cancellation, Ctrl-C shutdown, and channel closure. Without this hook, buffered rows could be lost.

## What lives in `carbon-core`

The generic ClickHouse runtime in `carbon-core` is split by responsibility:

- `config.rs` - endpoint, database, auth, source metadata, batching, and insert settings
- `http.rs` - minimal HTTP transport over `reqwest`
- `admin.rs` - explicit schema/bootstrap execution
- `rows/mod.rs` - row/table traits, row context, multi-row contract, and deterministic ID helpers
- `writer.rs` - per-buffer batch writer and background flush worker
- `processors.rs` - ClickHouse instruction/account processors
- `metrics.rs` - shared ClickHouse metrics for instruction and account families

Important boundary:

- Core knows how to talk to ClickHouse.
- Core knows how to buffer, flush, retry in memory, and shut down rows.
- Core does not know decoder-specific schema details.

## Write path and insert model

The sink uses client-side batching and HTTP `JSONEachRow` inserts.

Current behavior:

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

This matters for typed landing tables because one decoder wrapper can emit rows for many tables. A hot table should not force unrelated cold buffers to flush early, and a cold table should still flush when idle.

## Insert settings

Synchronous inserts are the default:

- `ClickHouseConfig::new(...)` defaults to `ClickHouseInsertSettings::Sync`.
- `ClickHouseConfig::from_database_url(...)` defaults to `ClickHouseInsertSettings::Sync`.
- Existing generated setup helpers remain sync by default.

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

The public sink API intentionally does not expose `wait_for_async_insert=0`. Fire-and-forget inserts are a poor default for this sink because they hide server-side insert failures from the Carbon process.

All insert requests include `date_time_input_format=best_effort`; async settings are merged into the same query-setting path.

## Current generated row scope

The current implemented scope is canary-limited but covers all three row families:

- Jupiter swap typed instruction landing rows.
- Jupiter swap typed CPI-event landing rows.
- Token Program typed account landing rows for mint, token, and multisig.
- Renderer templates for typed account, instruction, and CPI-event ClickHouse generation.
- `withClickHouse` feature-gated generated modules.

The production table model is one typed landing table per generated row family. This avoids the schema, compression, and query-shape problems of one large generic JSON landing table.

## Jupiter example

`examples/jupiter-swap-clickhouse` is a real-world smoke test for the Jupiter instruction and CPI-event path.

It validates:

- `RpcBlockCrawler` datasource
- production RPC URL loading through `.env`
- ClickHouse bootstrap through generated Jupiter migration helpers
- Jupiter instruction and CPI-event decoding
- typed ClickHouse row dispatch
- core writer batching and shutdown drain

It does not validate:

- Token Program account rows
- multi-decoder fan-in
- async insert settings
- replicated/distributed ClickHouse DDL

Use a bounded slot range when testing the example against production RPC.

## Token Program example

`examples/token-program-clickhouse` is a real-world smoke test for the account-family path.

It validates:

- filtered RPC `getProgramAccounts` or Helius gPA v2
- generated Token Program ClickHouse account table bootstrap
- Token Program account decoding from real mainnet account data
- generated token account landing rows
- `ClickHouseAccountProcessor`
- account-family metrics
- per-buffer writer flushing and shutdown drain

The example requires a token-account owner and/or mint filter so it does not accidentally fetch the entire Token Program account set. By default it is focused on token accounts because those can be bounded safely. The generated mint and multisig account rows remain covered by compile/tests and schema generation, but they are not the default real-world query path.

## Identity, replay, and table semantics

The current sink is landing-only and append-only.

Generated rows use deterministic IDs:

- instruction rows use `deterministic_instruction_id(...)`
- CPI-event rows use `deterministic_event_id(...)`
- account rows use `deterministic_account_id(...)`

That gives stable row identity across reprocessing, which is required for future canonicalization, deduplication, or stronger retry idempotency.

ClickHouse primary keys do not enforce uniqueness, so deterministic IDs are identity metadata, not online deduplication by themselves.

The sink does not currently do:

- online row-level deduplication
- serving-table resolution
- replay convergence
- coverage/range tracking
- durable retry state

So the present contract is stable landing identity, not full canonical warehouse semantics.

## Metrics and observability

The sink adds its own metrics because batch sinks need different visibility than row-at-a-time sinks.

Tracked today:

- inserted rows
- failed rows in failed batches
- current buffered row count
- successful flush batches
- failed flush batches
- flush duration histogram

Metrics are separated by processor family:

- `clickhouse.instructions.*`
- `clickhouse.accounts.*`

Background flushes and shutdown-triggered flushes record through the shared ClickHouse metrics module, not through processor-local foreground accounting.

## Generator state

The renderer now has ClickHouse support for:

- decoder manifest support for a `clickhouse` feature
- typed account landing row templates
- typed instruction landing row templates
- typed per-event CPI landing row templates
- generated account `clickhouse/mod.rs`
- generated instruction `clickhouse/mod.rs`
- generated Cargo dependencies for ClickHouse, `serde`, and `chrono`

The rollout is still canary-limited. The implementation is generator-backed, but the repository has not regenerated every decoder with ClickHouse output.

## Production notes

The expected scaled deployment is multiple Carbon processes running in parallel, each with its own datasource/decoder/processor pipeline and each writing to ClickHouse.

For that model:

- Each process owns local buffering and flush state.
- The writer flushes independently per `(table, partition)` buffer.
- Synchronous inserts remain the default for backfills and deterministic ingestion.
- Async-wait inserts are available for live production ingestion with many writers.
- ClickHouse table design is one typed landing table per generated row family.

Near-term production follow-ups:

- Add renderer-controlled replicated/distributed DDL modes.
- Add explicit insert deduplication token support if retry idempotency needs to be stronger than deterministic landing IDs.
