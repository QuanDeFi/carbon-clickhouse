## Carbon ClickHouse Sink

`clickhouse-upstream-v1` adds a ClickHouse landing-table sink that fits Carbon's existing processor architecture.

## What this branch adds

This branch does **not** add a full ClickHouse warehouse subsystem. It adds the landing-table foundation:

1. a small generic ClickHouse runtime inside `carbon-core`
2. a core lifecycle hook so buffered processors can flush on shutdown
3. typed instruction and CPI-event ClickHouse canary coverage for `jupiter-swap-decoder`
4. typed account ClickHouse canary coverage for `token-program-decoder`
5. renderer support for generated typed account, instruction, and CPI-event landing rows

## How it fits into Carbon

ClickHouse is implemented as normal Carbon processors, not as a new pipeline type.

The runtime shape is:

1. one or more datasources emit Carbon updates
2. Carbon builds decoded account, instruction, transaction, or block inputs
3. a ClickHouse-backed processor receives decoded inputs
4. decoder-owned code maps those inputs into typed ClickHouse row families
5. the generic ClickHouse batch writer groups rows by table and partition
6. batches are inserted into ClickHouse over HTTP with `JSONEachRow`

That choice keeps the ClickHouse path aligned with Carbon's existing model:

* pipelines route data
* processors own side effects
* decoder crates own schema and storage-specific row mapping

## The core architectural change: processor finalization

The most important Carbon-core change is processor finalization.

Because ClickHouse writes are buffered, accepted rows may still be in memory when the pipeline shuts down. This branch adds:

* `Processor::finalize()`
* pipe wrappers forwarding `finalize()`
* `Pipeline` calling `finalize_pipes()` on shutdown paths before exporter shutdown

That lets ClickHouse processors flush buffered rows deterministically on datasource cancellation, Ctrl-C shutdown, and channel closure. Without this hook, buffered rows could be lost.

## What lives in `carbon-core`

The generic ClickHouse runtime in `carbon-core`:

* `config.rs` - endpoint, database, auth, source metadata, and batch settings
* `http.rs` - minimal HTTP transport over `reqwest`
* `admin.rs` - explicit schema/bootstrap execution
* `rows/mod.rs` - row/table traits and deterministic row ID helpers
* `writer.rs` - buffered batch writer
* `processors.rs` - ClickHouse instruction/account processors and sink metrics

Important boundary:

* core knows how to talk to ClickHouse
* core knows how to buffer and flush rows
* core does not know any decoder-specific schema details

## Write path and insert model

The sink uses client-side batching and HTTP `JSONEachRow` inserts.

Current behavior:

* rows are buffered in memory
* buffers are grouped by `(table, partition_key())`
* size-based flush happens per buffer, not globally across all buffered rows
* a background timer flushes stale buffers whose own `flush_interval` has elapsed
* remaining buffers are drained in `finalize()`
* rows are serialized with `serde::Serialize`
* inserts go through ClickHouse HTTP, not the native protocol

Synchronous inserts remain the default because they are predictable for backfills and deterministic ingestion. Server-side async inserts can be enabled explicitly through `ClickHouseConfig::with_insert_settings(...)` for live production ingestion with many Carbon writers. The sink always uses `wait_for_async_insert=1` in async mode and does not expose fire-and-forget inserts.

## Current scope

The current implemented scope is canary-limited but no longer single-event-only:

* Jupiter swap typed instruction landing rows
* Jupiter swap typed CPI-event landing rows
* Token Program typed account landing rows for mint, token, and multisig
* renderer templates for typed account, instruction, and CPI-event ClickHouse generation
* `withClickHouse` feature-gated generated modules

The canary path has been validated with:

* `cargo check -p carbon-jupiter-swap-decoder --features clickhouse`
* `cargo check -p carbon-token-program-decoder --features clickhouse`
* `cargo check -p jupiter-swap-clickhouse-carbon-example`
* renderer tests for ClickHouse module and row generation

## Identity, replay, and table semantics

The current sink is landing-only and append-only.

Generated rows use deterministic IDs:

* instruction rows use `deterministic_instruction_id(...)`
* CPI-event rows use `deterministic_event_id(...)`
* account rows use `deterministic_account_id(...)`

That gives stable row identity across reprocessing, which is required for future canonicalization or dedup logic.

But the sink does not currently do:

* online row-level deduplication
* serving-table resolution
* replay convergence
* coverage/range tracking
* durable retry state

So the present contract is stable landing identity, not full canonical warehouse semantics.

## Metrics and observability

The sink adds its own metrics because batch sinks need different visibility than row-at-a-time sinks.

Tracked today:

* inserted rows
* failed rows in failed batches
* current buffered row count
* successful flush batches
* failed flush batches
* flush duration histogram

Metrics are separated by processor family:

* `clickhouse.instructions.*`
* `clickhouse.accounts.*`

## Generator state

The renderer now has ClickHouse support for:

* decoder manifest support for a `clickhouse` feature
* typed account landing row templates
* typed instruction landing row templates
* typed per-event CPI landing row templates
* generated account `clickhouse/mod.rs`
* generated instruction `clickhouse/mod.rs`
* generated Cargo dependencies for ClickHouse, `serde`, and `chrono`

The rollout is still canary-limited. The implementation is generator-backed, but the repository has not regenerated every decoder with ClickHouse output.

## Production notes

The production target is one typed landing table per generated row family. For multi-process Carbon deployments, each process owns local per-buffer batching and writes to the same ClickHouse table family.

Near-term production follow-ups:

* add renderer-controlled replicated/distributed DDL modes
* add explicit insert deduplication token support if retry idempotency needs to be stronger than deterministic landing IDs
