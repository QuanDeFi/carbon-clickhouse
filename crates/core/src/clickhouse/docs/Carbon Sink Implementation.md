## Carbon ClickHouse Sink Implementation

The ClickHouse sink is a generator-backed landing-table backend that fits Carbon's existing processor architecture. This document describes the current implementation, configuration, generated output, examples, validation commands, and operational checks.

For design rationale, architectural invariants, responsibility boundaries, and non-goals, see `ClickHouse Sink Architecture.md`.

The implementation is landing-only. It is not a warehouse serving layer, replay convergence system, or online deduplication system.

## Implementation Scope

The ClickHouse path includes:

1. A generic ClickHouse runtime inside `carbon-core`.
2. Processor finalization so buffered processors drain on shutdown.
3. Typed instruction and CPI-event ClickHouse coverage for `jupiter-swap-decoder`.
4. Typed account ClickHouse coverage for `jupiter-swap-decoder` TokenLedger and `token-program-decoder`.
5. Typed instruction ClickHouse coverage for `token-program-decoder`.
6. Renderer support for generated typed account, instruction, and CPI-event landing rows.
7. A per-buffer ClickHouse writer for independent Carbon processes writing into the same table families.
8. Explicit synchronous and async-wait insert settings, with synchronous inserts as the default.
9. Runtime controls for byte-aware batching, global backpressure, transport timeouts, gzip, retries, and default exact-batch insert deduplication tokens.
10. Renderer-controlled landing-table DDL modes for `MergeTree`, `ReplicatedMergeTree`, and `Distributed` deployments.
11. CLI parity for ClickHouse renderer options through `--clickhouse-options`.
12. Managed schema drift checks that safely repair enum-extension drift and reject unsafe live-table drift before ingestion.
13. RPC block crawler reliability hardening for real-world ClickHouse live examples that depend on finalized block streams.

## How It Fits Into Carbon

ClickHouse uses normal Carbon processors, not a separate pipeline type.

The runtime shape is:

1. One or more datasources emit Carbon updates.
2. Carbon builds decoded account or instruction inputs.
3. A ClickHouse-backed processor receives decoded inputs.
4. Decoder-owned code maps those inputs into typed ClickHouse row families.
5. The generic ClickHouse batch writer groups rows by table and partition.
6. Batches are inserted into ClickHouse over HTTP with `JSONEachRow`.

That keeps the ClickHouse path aligned with Carbon's existing model:

- Pipelines route data.
- Processors own side effects.
- Decoder crates own schema and storage-specific row mapping.

## Carbon Core And Datasource Touches

Most of the ClickHouse implementation lives in generated decoder modules and the
`carbon-core::clickhouse` runtime. The branch also touches a small number of
shared Carbon paths because a buffered ClickHouse sink depends on delivery and
shutdown behavior that happens before rows reach the sink.

Those shared touches are intentional:

- Processor finalization was added so buffered processors can drain rows on
  shutdown. Without this lifecycle hook, a ClickHouse processor could process
  decoded updates but exit before its in-memory buffers are flushed.
- Pipeline shutdown calls processor finalization before exporter shutdown so
  ClickHouse rows accepted by processors are drained before the process exits.
- The RPC block crawler was hardened because both real-world ClickHouse
  examples use finalized block streams. Sink-local retries cannot recover a
  transaction update that the datasource dropped before the processor saw it.

The RPC block crawler changes are reliability changes, not ClickHouse schema
logic:

- near-head temporary `getBlock` failures such as `-32004
  BlockNotAvailable` and `-32014 BlockStatusNotAvailableYet` are retried
  instead of treated as permanent skipped slots;
- permanent Solana skip conditions such as `-32001`, `-32007`, and `-32009`
  are still skipped and logged with the extracted RPC error code;
- downstream transaction delivery uses awaited `send(...)` instead of
  `try_send(...)`, so a full Carbon channel applies backpressure rather than
  silently dropping updates;
- the block fetcher and task processor shut down in an order that lets queued
  blocks drain before the task exits.

These changes keep the example data source compatible with the ClickHouse
sink's durability assumptions: once a transaction update is accepted by the
processor, the ClickHouse writer can buffer, retry, and drain it; before that
point, datasource behavior controls whether the update exists at all.

## Solana Schema And Validation Boundary

The ClickHouse sink derives typed landing rows from decoder-owned Solana schema, not from observed live payloads.

Decoder-owned row families map to Solana program artifacts:

- account layouts
- instruction layouts
- CPI/event layouts
- shared IDL/Codama-defined types

IDL/Codama schema is the source of truth for known program shapes. This keeps ClickHouse aligned with Carbon's generator model and avoids hand-maintained Borsh layouts or one-off per-decoder payload mappings.

Live RPC, Geyser, account, transaction, and log data are untrusted until decoded. Owner checks, data length checks, discriminators, instruction account arrangement, and event discriminator handling remain part of the decoder boundary before the sink can create typed rows.

CPI/event rows are generated through the instruction path because reliable indexer events can be emitted as CPI instruction data. Logs can truncate, so the ClickHouse event path is based on decoded event instruction data rather than assuming logs are complete event storage.

## Processor Finalization

Buffered ClickHouse writes require processor finalization. A processor can process rows that remain in memory until a batch threshold, timer, explicit flush, or shutdown drain fires.

The core lifecycle includes:

- `Processor::finalize()`
- pipe wrappers forwarding `finalize()`
- `Pipeline` calling `finalize_pipes()` on shutdown paths before exporter shutdown

ClickHouse processors call writer shutdown from `finalize()`, which drains all remaining buffers and stops the background worker. This preserves processed rows during datasource cancellation, Ctrl-C shutdown, and channel closure.

## What Lives In `carbon-core`

The generic ClickHouse runtime in `carbon-core` is split by responsibility:

- `config.rs` - endpoint, database, auth, source metadata, batching, transport, retry, deduplication, and insert settings
- `http.rs` - HTTP transport over `reqwest`, including query settings, gzip request bodies, and classified errors
- `admin.rs` - explicit schema/bootstrap execution, managed table reconciliation, and schema drift checks
- `rows/mod.rs` - row/table traits, row context, multi-row contract, and deterministic ID helpers
- `writer.rs` - per-buffer batch writer, byte backpressure, retry loop, and background flush worker
- `processors.rs` - `ClickHouseInstructionProcessor` and `ClickHouseAccountProcessor`
- `metrics.rs` - shared ClickHouse metrics for all ClickHouse processor families

Important boundary:

- Core knows how to talk to ClickHouse.
- Core knows how to buffer, flush, retry in memory, and shut down rows.
- Core does not know decoder-specific schema details.

Rows receive sink metadata through `ClickHouseRowContext`, which contains `source_name`, `mode`, and `decoder_version`.

Schema/bootstrap execution uses:

- `ClickHouseSchema::managed_tables(config) -> Vec<ClickHouseManagedTable>` for generated managed tables with expected column definitions.
- `ClickHouseAdmin::execute_query(...)` for a single query.
- `ClickHouseAdmin::execute_queries(...)` for explicit ordered query lists.
- `ClickHouseAdmin::execute_schema::<S>()` for schema bundles that implement `ClickHouseSchema`.

Schema execution uses the same `ClickHouseConfig` endpoint, database, auth, and HTTP client settings as data inserts. Schema queries include `date_time_input_format=best_effort`.

`ClickHouseAdmin::execute_schema::<S>()` creates missing managed tables, adds missing generated columns, validates live table layout, and compares live ClickHouse column types against generated expected types before rows are inserted.

Schema reconciliation is intentionally fixed and opinionated:

- Missing generated tables are created.
- Missing generated columns are added.
- Enum-extension-only drift is repaired with `ALTER TABLE ... MODIFY COLUMN` when all existing enum values and numeric IDs are preserved.
- Unsafe drift fails fast. That includes scalar type changes, tuple shape changes, removed enum values, changed enum numeric IDs, engine drift, partition drift, order-key drift, and any unclassified mismatch.

The ingestion path does not expose a drop/recreate policy. Destructive table repair belongs in explicit dev/admin operations, not Carbon process startup.

## What Lives In Decoder Crates

Decoder crates own concrete ClickHouse schemas and row mapping. The current committed generated output is canary-limited.

Jupiter swap instruction and CPI/event canary modules:

- `decoders/jupiter-swap-decoder/src/instructions/clickhouse/mod.rs`
- `decoders/jupiter-swap-decoder/src/instructions/clickhouse/*_row.rs`
- `decoders/jupiter-swap-decoder/src/instructions/clickhouse/*_event_row.rs`

Jupiter swap account canary modules:

- `decoders/jupiter-swap-decoder/src/accounts/clickhouse/mod.rs`
- `decoders/jupiter-swap-decoder/src/accounts/clickhouse/token_ledger_row.rs`

Token Program account canary modules:

- `decoders/token-program-decoder/src/accounts/clickhouse/mod.rs`
- `decoders/token-program-decoder/src/accounts/clickhouse/mint_row.rs`
- `decoders/token-program-decoder/src/accounts/clickhouse/multisig_row.rs`
- `decoders/token-program-decoder/src/accounts/clickhouse/token_row.rs`

Token Program instruction canary modules:

- `decoders/token-program-decoder/src/instructions/clickhouse/mod.rs`
- `decoders/token-program-decoder/src/instructions/clickhouse/*_row.rs`

Those modules provide:

- typed row structs
- table names and DDL
- managed table metadata and generated expected column definitions
- decoder-family wrapper types
- `ClickHouseRows` implementations
- setup helpers used by examples

The committed Jupiter and Token Program canary modules bootstrap tables through
generated managed table metadata. Each row module exposes the generated `CREATE
TABLE` SQL and the expected column definitions used for additive
`ALTER TABLE ... ADD COLUMN IF NOT EXISTS` operations and drift validation. The
renderer does not generate destructive type-change migrations.

Examples should call these generated helpers. They should not duplicate decoder-specific schema or row mapping logic.

Generated setup helpers use decoder defaults:

- `clickhouse_config_from_database_url(...)` builds a `ClickHouseConfig` from `DATABASE_URL`.
- `bootstrap_clickhouse_from_database_url(...)` executes the decoder's generated managed schema reconciliation and returns the config.
- `clickhouse_processor(config)` constructs the generated processor alias.
- `setup_clickhouse(...)` bootstraps schema and returns the generated processor in one call.

Callers that need production settings should use `bootstrap_clickhouse_from_database_url(...)`, then apply `ClickHouseConfig` builders before constructing the processor.

`DATABASE_URL` provides the HTTP endpoint and optional auth credentials. Generated helpers pass their own database default, currently `DEFAULT_DATABASE = "default"`, instead of deriving the database from the URL path.

## End-User Setup Flow

For a generated decoder with ClickHouse enabled, the normal setup is:

1. Generate or scaffold the decoder with `--with-clickhouse true`, or pass
   production DDL options through `--clickhouse-options <jsonOrFile>`.
2. Enable the generated decoder crate's `clickhouse` feature in the consuming
   application.
3. Build a `ClickHouseConfig` from `DATABASE_URL` with the generated
   `clickhouse_config_from_database_url(...)` or
   `bootstrap_clickhouse_from_database_url(...)` helper.
4. Run the generated migration/setup helper before ingestion. This creates
   missing generated tables, adds missing generated columns, repairs safe enum
   extensions, and fails on unsafe drift.
5. Apply any production runtime settings with `ClickHouseConfig` builder
   methods.
6. Attach the generated ClickHouse processor to the normal Carbon account or
   instruction pipe.
7. Let Carbon pipeline shutdown call processor `finalize()`, or call
   `finalize()` directly in custom wiring, so buffered rows drain.

Minimal example environment:

```env
DATABASE_URL=http://user:password@clickhouse-host:8123
RPC_URL=<provider-rpc-url>
PROMETHEUS_METRICS_ADDR=0.0.0.0:9464
LOG_LEVEL=info
```

The committed examples also support:

- `CLICKHOUSE_ASYNC_INSERT=true` to opt into async-wait inserts.
- Jupiter only: `BLOCK_CRAWLER_START_SLOT`, `BLOCK_CRAWLER_END_SLOT`, and
  `BLOCK_CRAWLER_HEAD_LAG_SLOTS` for bounded, catch-up, or head-follow modes.

Do not put multiple independent Carbon processes on the same
`PROMETHEUS_METRICS_ADDR` port on one host. Run each process with its own
metrics port or let the process supervisor/container platform expose distinct
targets.

## Runtime Configuration Reference

`ClickHouseConfig` is the runtime contract between generated decoder helpers
and the generic writer.

| Field or builder | Default | Purpose |
| --- | --- | --- |
| `endpoint` | From `DATABASE_URL` scheme, host, and port | ClickHouse HTTP endpoint. |
| `database` | Generated helper default, currently `default` | Target database passed as the ClickHouse `database` query parameter. |
| `username` / `password` | From `DATABASE_URL` userinfo | Optional HTTP basic auth. |
| `table` | Generated default table name | Compatibility field for single-table helpers; multi-table generated rows choose concrete tables through `ClickHouseRow::table_name()`. |
| `source_name` | Generated helper default, often decoder/example specific | Row metadata identifying the datasource or pipeline source, for example `rpc_block_crawler` or `rpc_get_multiple_accounts`. |
| `mode` | Generated helper default, then example-specific override | Row metadata such as `backfill`, `live`, or `snapshot`. |
| `decoder_version` | Generated helper default, currently logical `v1` | Row metadata for logical decoder/version routing; not currently a crate version or schema hash. |
| `with_insert_settings(...)` | `ClickHouseInsertSettings::Sync` | Select synchronous inserts or async-wait inserts. |
| `with_batch_settings(...)` | Required generated `max_rows` and `flush_interval`; byte/global caps unset | Controls per-buffer row threshold, per-buffer byte threshold, global buffered row/byte caps, and stale flush interval. |
| `with_transport_settings(...)` | No explicit timeouts, no compression, default reqwest pool | Controls request/connect/pool timeouts, max idle connections per host, gzip body compression, and user agent. |
| `with_retry_settings(...)` | `max_retries = 3`, `initial_backoff = 100ms`, `max_backoff = 5s`, `jitter = true` | Controls transient HTTP retry behavior. |
| `with_deduplication_settings(...)` | `ClickHouseDeduplicationSettings::ExactBatchHash` | Emits a stable `insert_deduplication_token` per exact table/query/body batch unless disabled. |

The examples expose only a thin environment surface. More advanced runtime
settings are Rust API configuration, not environment variables, so production
applications should configure them in their own pipeline wiring.

## Write Path And Insert Model

The sink uses client-side batching and HTTP `JSONEachRow` inserts.

Writer behavior:

- Rows are serialized before buffering so row and byte limits match the exact HTTP body.
- Insert targets come from each row's `ClickHouseRow::table_name()`, so generated multi-table wrappers choose the concrete landing table per row.
- Buffers are keyed by `(table, partition_key())`.
- Row-count and byte-threshold flushes happen per buffer, not globally across all buffered rows.
- Global buffered row and byte caps can reject new rows after one local drain attempt over stale and largest buffers.
- A background worker starts lazily on the first async writer operation.
- The background timer flushes only stale buffers whose own `last_flush` exceeds `flush_interval`.
- `flush()` drains all buffers for explicit/manual drains.
- `shutdown()` drains all buffers and stops the background worker.
- Processors call `shutdown()` from `finalize()`.
- Retryable flush failures use configured exponential backoff.
- Failed batches are reinserted into the same buffer after the final retry and surfaced on the next writer operation or shutdown.
- Rows are serialized with `serde::Serialize`.
- Inserts go through ClickHouse HTTP, not the native protocol.
- Insert SQL is `INSERT INTO {table} FORMAT JSONEachRow`; the writer does not send an explicit column list.

This matters for typed landing tables because one decoder wrapper can emit rows for many tables. A hot table does not force unrelated cold buffers to flush early, and a cold table still flushes when idle.

Writer APIs exposed by `carbon-core::clickhouse`:

- `ClickHouseBatchWriter::buffer_row(...)` returns `ClickHouseBufferOutcome::Buffered` or `ClickHouseBufferOutcome::Flushed`.
- `ClickHouseBatchWriter::flush()` explicitly drains all buffers and returns `ClickHouseFlushOutcome`.
- `ClickHouseBatchWriter::shutdown()` stops the background worker and drains all buffers.
- `ClickHouseBatchWriter::snapshot()` returns `ClickHouseWriterSnapshot` with buffered rows, buffered bytes, active buffers, and any retained background error.

The snapshot API is for debugging and tests. Production monitoring should use Carbon metrics and ClickHouse server-side system tables.

Runtime buffer partition keys are generated by each row's `partition_key()` method:

- instruction and CPI/event rows use the year extracted from `partition_time`
- account rows use `partition_slot`, computed as `slot / 1_000_000`

This key controls in-process buffer grouping. It is related to, but not the same API as, renderer-controlled ClickHouse `PARTITION BY` DDL. Production DDL can override ClickHouse table partitioning while the writer still groups buffered rows by the row's generated runtime partition key.

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

Each insert attempt also includes a generated `query_id`:

```text
carbon-clickhouse-{table}-{sequence}-{attempt}
```

The query ID is for ClickHouse query-log traceability only. It is not used for deduplication.

`ClickHouseDeduplicationSettings::ExactBatchHash` is the default. When enabled,
each exact batch includes:

```text
insert_deduplication_token = sha256(table + "\n" + insert_query + "\n" + exact_body)
```

Retry behavior is controlled by `ClickHouseRetrySettings`. The sink retries network errors, request timeouts, HTTP `408`, `429`, `5xx`, and ClickHouse "too many parts" / "too many inactive parts" responses. Schema errors, auth errors, malformed requests, and most other `4xx` responses are treated as permanent.

## Multi-Writer Behavior

Production deployments can run many Carbon processes in parallel, each with its
own datasource, decoder, processor, and local ClickHouse writer. There is no
global writer coordinator in `carbon-core::clickhouse`.

The integration gives ClickHouse enough request and row metadata to operate
safely in that model:

- `source_name`, `mode`, and `decoder_version` are written into every landing
  row so downstream queries can separate pipelines, sources, backfills,
  snapshots, and live ingestion modes.
- Each insert attempt sends a `query_id` in the form
  `carbon-clickhouse-{table}-{sequence}-{attempt}` so operators can trace
  Carbon inserts in `system.query_log` and async-insert logs. This is trace
  metadata, not a stable writer identity.
- `ClickHouseDeduplicationSettings::ExactBatchHash` sends
  `insert_deduplication_token` based on the exact table, insert query, and
  body. This gives retry idempotency for the exact same batch. It does not
  deduplicate different batch boundaries or replayed landing rows.
- Async-wait mode sends `async_insert=1` and `wait_for_async_insert=1`, allowing
  ClickHouse to coalesce inserts server-side while the Carbon process still
  observes insert success or failure.
- Renderer-generated local `MergeTree` DDL includes
  `non_replicated_deduplication_window = 1000` by default when local
  non-replicated deduplication is relevant.
- Replicated and distributed table topology is controlled by renderer DDL
  options, not by the runtime writer.

For scaled live ingestion, use async-wait inserts, consistent DDL across all
writers, unique `source_name` values when operators need per-pipeline
separation, and external process/container labels for per-process Prometheus
views. For bounded backfills and deterministic replay jobs, synchronous inserts
remain the simpler default.

## Production Config Examples

Generated decoder setup helpers keep synchronous inserts by default. Production callers can modify the returned `ClickHouseConfig` before constructing the processor.

### Synchronous Backfill

Use synchronous inserts for bounded crawls, deterministic backfills, and replay jobs where the Carbon process should observe insert success before moving on.

```rust
use std::time::Duration;

use carbon_core::clickhouse::{
    ClickHouseBatchSettings,
    ClickHouseHttpCompression,
    ClickHouseRetrySettings,
    ClickHouseTransportSettings,
};

let config = bootstrap_clickhouse_from_database_url(&database_url).await?
    .with_batch_settings(ClickHouseBatchSettings {
        max_rows: 50_000,
        max_bytes: Some(25 * 1024 * 1024),
        max_buffered_rows: Some(500_000),
        max_buffered_bytes: Some(512 * 1024 * 1024),
        flush_interval: Duration::from_secs(10),
    })
    .with_transport_settings(ClickHouseTransportSettings {
        request_timeout: Some(Duration::from_secs(60)),
        connect_timeout: Some(Duration::from_secs(10)),
        pool_idle_timeout: Some(Duration::from_secs(30)),
        pool_max_idle_per_host: Some(16),
        compression: ClickHouseHttpCompression::Gzip,
        user_agent: Some("carbon-clickhouse-backfill".to_string()),
    })
    .with_retry_settings(ClickHouseRetrySettings {
        max_retries: 3,
        initial_backoff: Duration::from_millis(250),
        max_backoff: Duration::from_secs(10),
        jitter: true,
    });
```

`ExactBatchHash` is enabled by default and emits an `insert_deduplication_token`
derived from the exact table, insert query, and body. Callers can disable it
through `with_deduplication_settings(ClickHouseDeduplicationSettings::Disabled)`
only when duplicate retry inserts in raw landing tables are acceptable.

### Async-Wait Live Ingestion

Use async-wait inserts for live deployments with many Carbon writers, where ClickHouse should coalesce inserts server-side but the Carbon process should still wait for server acknowledgement.

```rust
use std::time::Duration;

use carbon_core::clickhouse::{
    ClickHouseAsyncInsertSettings,
    ClickHouseBatchSettings,
    ClickHouseHttpCompression,
    ClickHouseInsertSettings,
    ClickHouseRetrySettings,
    ClickHouseTransportSettings,
};

let config = bootstrap_clickhouse_from_database_url(&database_url).await?
    .with_insert_settings(ClickHouseInsertSettings::AsyncWait(
        ClickHouseAsyncInsertSettings {
            busy_timeout_ms: Some(1_000),
            max_data_size: Some(16 * 1024 * 1024),
            max_query_number: Some(64),
            deduplicate: Some(true),
        },
    ))
    .with_batch_settings(ClickHouseBatchSettings {
        max_rows: 5_000,
        max_bytes: Some(5 * 1024 * 1024),
        max_buffered_rows: Some(100_000),
        max_buffered_bytes: Some(128 * 1024 * 1024),
        flush_interval: Duration::from_secs(1),
    })
    .with_transport_settings(ClickHouseTransportSettings {
        request_timeout: Some(Duration::from_secs(30)),
        connect_timeout: Some(Duration::from_secs(5)),
        pool_idle_timeout: Some(Duration::from_secs(30)),
        pool_max_idle_per_host: Some(32),
        compression: ClickHouseHttpCompression::Gzip,
        user_agent: Some("carbon-clickhouse-live".to_string()),
    })
    .with_retry_settings(ClickHouseRetrySettings {
        max_retries: 5,
        initial_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(5),
        jitter: true,
    });
```

The sink intentionally does not expose `wait_for_async_insert=0`.

## Generated Row Scope

The generated row scope is canary-limited and covers account, instruction, and CPI-event row families:

- Jupiter swap typed instruction landing rows.
- Jupiter swap typed CPI-event landing rows.
- Jupiter swap typed TokenLedger account landing rows.
- Token Program typed account landing rows for mint, token, and multisig.
- Token Program typed instruction landing rows.
- Renderer templates for typed account, instruction, and CPI-event ClickHouse generation.
- `withClickHouse` feature-gated generated modules.
- `withClickHouse: true` for default `MergeTree` DDL.
- `withClickHouse: { ... }` for renderer-controlled DDL options.

The table model is typed landing tables: one table per instruction family, one
table per account family, and one table per generated CPI/event family. Each
CPI/event table keeps the common event metadata plus the typed payload columns
for that specific event family instead of a generic JSON payload.

This is the current shipping model after the CPI/event table reversal in
`3f98a52f9`. The earlier temporary Postgres-aligned model from `9c3ef9444`
used one unified `jupiter_swap_cpi_event_landing` table. That table is obsolete
local state now; generated Jupiter CPI/event rows are emitted to separate
per-event tables.

Generated row mapping derives structured ClickHouse types from the decoder schema:

- primitives map to native ClickHouse scalar types
- `u128` and `i128` use generated serializer wrappers while retaining `UInt128` and `Int128` columns
- arrays and fixed arrays map to `Array(...)`
- structs and tuples map to generated Rust helper structs and ClickHouse `Tuple(...)`
- fieldless enums map to `Enum8` or `Enum16`
- payload enums map to generated tagged-union helper structs with a variant enum and typed payload fields

Known decoder-owned enum payloads are not stringified. JSON remains only as an explicit fallback for unsupported dynamic shapes.

Unsupported known decoder schema should fail generation unless `allowClickHouseJsonFallback` is explicitly enabled. This keeps accidental type loss visible during renderer validation instead of hiding it behind a generic JSON column.

The fallback can be enabled either as the top-level renderer option `allowClickHouseJsonFallback: true` or as `withClickHouse: { allowJsonFallback: true }`. The committed canary decoder output is generated without JSON fallback.

## Generated Landing Row Contract

Generated table names follow these patterns:

- instruction rows: `{program}_{instruction}_instruction_landing`
- CPI/event rows: `{program}_{event}_landing`
- account rows: `{program}_{account}_account_landing`

Current canary table families include:

- Jupiter instruction tables such as `jupiter_swap_route_instruction_landing`.
- Jupiter CPI/event tables such as `jupiter_swap_fee_event_landing`, `jupiter_swap_swap_event_landing`, and `jupiter_swap_swaps_event_landing`.
- Token Program account tables: `token_program_mint_account_landing`, `token_program_multisig_account_landing`, and `token_program_token_account_landing`.
- Token Program instruction tables such as `token_program_transfer_checked_instruction_landing`, `token_program_initialize_account3_instruction_landing`, and `token_program_sync_native_instruction_landing`.

Common instruction columns:

- `program_id`
- `family_name`
- `instruction_type`
- `instruction_id`
- `slot`
- `signature`
- `instruction_index`
- `stack_height`
- `absolute_path`
- `source_name`
- `mode`
- `decoder_version`
- `ingest_ts`
- `chain_time`
- `partition_time`
- `block_hash`
- `tx_index`

Common CPI/event columns are the instruction metadata above, replacing instruction identity fields with:

- `event_type`
- `event_id`
- `event_seq`

Common account columns:

- `program_id`
- `family_name`
- `account_type`
- `account_id`
- `slot`
- `pubkey`
- `transaction_signature`
- `lamports`
- `owner`
- `executable`
- `rent_epoch`
- `source_name`
- `mode`
- `decoder_version`
- `ingest_ts`
- `partition_slot`

Payload columns are appended after the common columns and are generated from decoder-owned schema through the ClickHouse row mapper.

## Renderer DDL Modes

Generated row families support these DDL modes:

- `merge-tree` - default `MergeTree` landing table.
- `replicated-merge-tree` - `ReplicatedMergeTree` with configurable ZooKeeper/Keeper path, replica name, and optional `ON CLUSTER`.
- `distributed` - generated local table plus a `Distributed` table pointing at that local table.

The renderer option is backward-compatible:

```ts
renderVisitor(outputDir, {
    withClickHouse: true,
});
```

The renderer also accepts `withClickHouse: { enabled: false }` to keep object-shaped configuration while disabling ClickHouse generation.

Production DDL options use an object:

```ts
renderVisitor(outputDir, {
    withClickHouse: {
        ddlMode: 'replicated-merge-tree',
        onCluster: 'production_cluster',
        partitionBy: {
            instruction: 'toYYYYMM(partition_time)',
            event: 'toYYYYMM(partition_time)',
            account: 'partition_slot',
        },
        orderBy: {
            instruction: ['program_id', 'family_name', 'slot', 'instruction_id'],
            event: ['program_id', 'family_name', 'slot', 'event_id'],
            account: ['program_id', 'family_name', 'pubkey', 'slot'],
        },
        ttl: {
            instruction: 'partition_time + INTERVAL 365 DAY',
            event: 'partition_time + INTERVAL 365 DAY',
        },
        engineSettings: {
            index_granularity: 8192,
        },
        columnCodecs: {
            signature: 'ZSTD(3)',
            instruction_id: 'ZSTD(3)',
            account_id: 'ZSTD(3)',
        },
    },
});
```

Renderer-generated schema metadata creates tables and then exposes additive
`ALTER TABLE ... ADD COLUMN IF NOT EXISTS` operations for generated columns.
The same metadata is used for pre-ingest drift validation, so generated DDL and
runtime schema checks stay aligned. The renderer does not generate destructive
type-change migrations.

For `distributed` DDL mode, generated managed schema metadata creates and reconciles the local table first, then creates and reconciles the distributed table. The distributed table omits local `MergeTree` clauses because storage lives in the generated local table. Additive column operations are emitted for both local and distributed tables.

Column codecs are applied by column name to both common metadata columns and payload columns wherever the name matches `columnCodecs`.

Default DDL option values:

- `ddlMode`: `merge-tree`
- account `PARTITION BY`: `partition_slot`
- instruction/event `PARTITION BY`: `toYear(partition_time)`
- account `ORDER BY`: `(program_id, family_name, account_id, slot)`
- instruction `ORDER BY`: `(program_id, family_name, instruction_id, slot)`
- event `ORDER BY`: `(program_id, family_name, event_id, slot)`
- replicated table path: `/clickhouse/tables/{shard}/{database}/{table_name}`
- replicated replica name: `{replica}`
- distributed local table suffix: `_local`
- distributed local table engine: `replicated-merge-tree`
- distributed sharding key: `rand()`

The TypeScript renderer API supports the object form above. The CLI supports
the same options through `--clickhouse-options <jsonOrFile>` on both `parse`
and `scaffold`, while preserving `--with-clickhouse <boolean>` for the simple
default path.

## Jupiter Example

`examples/jupiter-swap-clickhouse` is the real-world smoke test for the Jupiter
instruction, CPI-event, and live TokenLedger account path.

It validates:

- `RpcBlockCrawler` datasource
- production RPC URL loading through `.env`
- ClickHouse bootstrap through generated Jupiter instruction/event and account migration helpers
- Jupiter instruction and CPI-event decoding
- Jupiter TokenLedger account decoding in pure head-follow mode
- typed ClickHouse row dispatch
- structured CPI-event payloads such as `route_plan.swap`
- core writer batching and shutdown drain
- `ShutdownStrategy::Immediate` with a bounded block range
- conservative block fetch concurrency while leaving Carbon and block-crawler channels at their upstream default size of `1_000`

Mode is inferred from slot environment variables:

- `BLOCK_CRAWLER_START_SLOT` and `BLOCK_CRAWLER_END_SLOT` set: bounded backfill, instruction/CPI-event path only.
- `BLOCK_CRAWLER_START_SLOT` set and `BLOCK_CRAWLER_END_SLOT` empty: catch-up then tail head, instruction/CPI-event path only.
- both slot envs empty: pure head-follow mode, instruction/CPI-event path plus live TokenLedger account fetching.

The TokenLedger account path fetches current confirmed account state with
`getAccountInfo` when a first-seen TokenLedger pubkey appears in live decoded
Jupiter instructions. It is intentionally disabled for historical backfills
because standard Solana RPC does not provide historical account state at an
arbitrary old slot.

It does not validate:

- Token Program account or instruction rows
- multi-decoder fan-in
- replicated/distributed ClickHouse DDL

Use a bounded slot range when testing the example against production RPC.

## Token Program Example

`examples/token-program-clickhouse` is the real-world smoke test for the Token
Program account and instruction path.

It validates:

- standard Solana JSON-RPC `getMultipleAccounts` against a fixed USDC account set
- generated Token Program ClickHouse account table bootstrap
- generated Token Program ClickHouse instruction table bootstrap
- Token Program account decoding from real mainnet account data
- generated mint, multisig, and token account landing rows
- live finalized block crawling for Token Program instruction rows
- generated Token Program instruction landing rows, including high-volume
  `TransferChecked`, ATA setup, WSOL lifecycle, and lower-frequency authority
  or mint/burn operations when they occur in the stream
- `ClickHouseAccountProcessor`
- `ClickHouseInstructionProcessor`
- account-family metrics
- instruction-family metrics
- per-buffer writer flushing and shutdown drain
- optional `CLICKHOUSE_ASYNC_INSERT=true`
- Prometheus scrape exposure on `PROMETHEUS_METRICS_ADDR`, default `0.0.0.0:9465`

The example is intentionally opinionated. It fetches four hardwired USDC
accounts: the USDC mint, the USDC mint-authority multisig, the USDC
freeze-authority multisig, and one USDC token holding account. It does not scan
Token Program accounts, does not use GPA pagination, does not require Helius
specific APIs, and has no `--source` mode. Account rows are current account
snapshots from the RPC response slot, so the account processor writes
`mode = snapshot` and `source_name = rpc_get_multiple_accounts`.

After the account snapshot, the example starts the RPC block crawler at the
next finalized slot and keeps running until interrupted. Instruction rows are
block-wide Token Program coverage from finalized blocks, so the instruction
processor writes `mode = live` and `source_name = rpc_block_crawler`.

## Identity, Replay, And Table Semantics

The sink is landing-only and append-only.

Generated rows use deterministic IDs:

- instruction rows use `deterministic_instruction_id(...)`
- CPI-event rows use `deterministic_event_id(...)`
- account rows use `deterministic_account_id(...)`

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
- inserted bytes
- failed rows in failed batches
- failed bytes in failed batches
- buffered row count
- buffered byte count
- active buffer count
- successful flush batches
- failed flush batches
- flush duration histogram
- retry count
- backpressure rejection count

Metrics are separated by processor family:

- `clickhouse.instructions.*`
- `clickhouse.accounts.*`

Background flushes and shutdown-triggered flushes record through the shared ClickHouse metrics module, not through processor-local foreground accounting.

## Generator State

The renderer supports:

- decoder manifest support for a `clickhouse` feature
- typed account landing row templates
- typed instruction landing row templates
- typed per-event CPI/event landing row templates
- generated account `clickhouse/mod.rs`
- generated instruction `clickhouse/mod.rs`
- generated Cargo dependencies for ClickHouse, `serde`, and `chrono`
- strict-by-default ClickHouse schema mapping with explicit `allowClickHouseJsonFallback`
- renderer-controlled DDL planning in `packages/renderer/src/clickhouseDdl.ts`
- CLI option forwarding for `--clickhouse-options <jsonOrFile>`

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
scripts/validate-clickhouse-decoder-rollout.sh
git diff --check
cargo fmt --all
```

For a broader non-committing decoder scan:

```bash
scripts/validate-clickhouse-decoder-rollout.sh --compile-all
```

For temporary regeneration checks from local IDL files:

```bash
scripts/validate-clickhouse-decoder-rollout.sh \
  --skip-renderer \
  --regenerate-idl-dir examples/versioned-decoders/idls \
  --regenerate-limit 2
```

Use `--regenerate-standard anchor` or `--regenerate-standard codama` to select the IDL standard for those temporary regeneration checks. Use `--skip-regenerated-compile` only when the goal is to inspect generated files without compiling the temporary crates.

Regeneration checks run the built Carbon CLI and require Node 20+ because current CLI dependencies require it. In environments where `node` is older, set `NODE_BIN`:

```bash
NODE_BIN=/path/to/node20 \
  scripts/validate-clickhouse-decoder-rollout.sh \
  --skip-renderer \
  --regenerate-idl-dir examples/versioned-decoders/idls \
  --regenerate-limit 2
```

For temporary regeneration checks from the root README decoder table, pass an RPC URL so the CLI can fetch on-chain Anchor IDLs for the listed program addresses:

```bash
scripts/validate-clickhouse-decoder-rollout.sh \
  --skip-renderer \
  --regenerate-from-readme \
  --rpc-url "$RPC_URL" \
  --regenerate-limit 5
```

The script scans every decoder crate, verifies that committed ClickHouse output remains canary-limited unless `--allow-broad-clickhouse` is set, and compiles any decoder package that currently exposes a `clickhouse` feature. Opt-in regeneration checks write generated ClickHouse-enabled decoders only to a temporary directory and patch those temporary crates to compile against the local `carbon-core`; the script fails if the repository working tree changes.

## Production Model

The scaled deployment model is multiple Carbon processes running in parallel, each with its own datasource/decoder/processor pipeline and each writing to ClickHouse.

For that model:

- Each process owns local buffering and flush state.
- The writer flushes independently per `(table, partition)` buffer.
- Synchronous inserts are the default for backfills and deterministic ingestion.
- Async-wait inserts are available for live production ingestion with many writers.
- `query_id` gives ClickHouse query-log traceability per insert attempt.
- Exact-batch deduplication tokens are emitted by default for retry
  idempotency of the same serialized batch.
- Row metadata (`source_name`, `mode`, `decoder_version`) is the application
  layer way to distinguish pipelines and ingestion modes in landing tables.
- ClickHouse table design is typed landing tables: one per instruction family, one per account family, and one per generated CPI/event family.

The sink does not register writer processes in ClickHouse and does not provide
a cross-process coordinator. Coordination across many Carbon processes belongs
to the process supervisor, deployment platform, or external control plane.

## Responsibility Split

The sink handles:

- typed row conversion from decoder-owned account, instruction, and CPI-event schemas
- deterministic landing IDs
- local row/byte batching and per-buffer flushing
- shutdown drain through processor finalization
- sync insert defaults and async-wait insert settings
- transient HTTP retry/backoff and failed-buffer preservation
- default exact-batch insert deduplication tokens
- generated managed schema bootstrap and safe drift reconciliation for landing
  tables
- sink-side metrics

ClickHouse handles:

- physical storage, compression, partitions, sorting, and merges
- replicated and distributed table topology
- server-side async insert batching when configured
- materialized views and downstream serving tables
- query-time serving performance and retention policies
- eventual dedup or canonicalization engines above landing tables, if chosen

External control-plane or application code handles:

- durable queues and durable retry state
- slot/range coverage tracking
- replay orchestration and source checkpoints
- DLQs and poison-record policy
- Solana finality and reorg policy
- schema rollout coordination across clusters
- serving API semantics

Those control-plane concerns intentionally do not live inside `carbon-core::clickhouse`.

## Monitoring Queries

These queries are operational starting points. Replace `default` and table patterns with the deployment database/table names.

Pending async insert queue:

```sql
SELECT
    database,
    table,
    count() AS queue_entries,
    sum(length(`entries.query_id`)) AS queued_queries,
    formatReadableSize(sum(total_bytes)) AS queued_bytes,
    min(first_update) AS oldest_entry
FROM system.asynchronous_inserts
WHERE database = 'default'
GROUP BY database, table
ORDER BY sum(total_bytes) DESC;
```

Async insert flush failures:

```sql
SYSTEM FLUSH LOGS;

SELECT
    event_time,
    database,
    table,
    status,
    bytes,
    query_id,
    flush_query_id,
    exception
FROM system.asynchronous_insert_log
WHERE database = 'default'
  AND event_time > now() - INTERVAL 1 HOUR
  AND status != 'Ok'
ORDER BY event_time DESC
LIMIT 100;
```

Part pressure by landing table:

```sql
SELECT
    database,
    table,
    count() AS active_parts,
    sum(rows) AS rows,
    formatReadableSize(sum(bytes_on_disk)) AS bytes_on_disk
FROM system.parts
WHERE active
  AND database = 'default'
  AND table LIKE '%_landing%'
GROUP BY database, table
ORDER BY active_parts DESC
LIMIT 50;
```

Active merges:

```sql
SELECT
    database,
    table,
    elapsed,
    progress,
    num_parts,
    formatReadableSize(total_size_bytes_compressed) AS compressed_bytes
FROM system.merges
WHERE database = 'default'
ORDER BY elapsed DESC;
```

Insert failures in the query log:

```sql
SYSTEM FLUSH LOGS;

SELECT
    event_time,
    type,
    query_id,
    query_kind,
    exception_code,
    exception,
    query
FROM system.query_log
WHERE event_time > now() - INTERVAL 1 HOUR
  AND query_kind = 'Insert'
  AND type IN ('ExceptionBeforeStart', 'ExceptionWhileProcessing')
ORDER BY event_time DESC
LIMIT 100;
```

Insert throughput:

```sql
SYSTEM FLUSH LOGS;

SELECT
    toStartOfMinute(event_time) AS minute,
    sum(written_rows) AS rows,
    formatReadableSize(sum(written_bytes)) AS bytes,
    count() AS inserts
FROM system.query_log
WHERE event_time > now() - INTERVAL 1 HOUR
  AND type = 'QueryFinish'
  AND query_kind = 'Insert'
  AND query LIKE 'INSERT INTO%'
GROUP BY minute
ORDER BY minute DESC;
```

## Rollout Policy

Committed decoder output stays canary-limited:

- Jupiter swap validates instruction and CPI-event ClickHouse rows.
- Token Program validates account and instruction ClickHouse rows.
- Other decoders stay without committed ClickHouse modules until broader decoder validation is intentionally rolled out.

Broad decoder validation is done without committing generated output. Use `scripts/validate-clickhouse-decoder-rollout.sh` for current canary validation, `--compile-all` for a broader baseline compile scan, `--regenerate-idl-dir` for local IDL files, and `--regenerate-from-readme --rpc-url "$RPC_URL"` for README-listed program IDs. For broader `withClickHouse` regeneration, use a temporary branch or worktree, compile with `--allow-broad-clickhouse`, and commit only the intended rollout set.
