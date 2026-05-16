## Carbon ClickHouse Sink Implementation

The ClickHouse sink is a generator-backed landing-table backend that fits Carbon's existing processor architecture. This document describes the current implementation, configuration, generated output, examples, validation commands, and operational checks.

For design rationale, architectural invariants, responsibility boundaries, and non-goals, see `ClickHouse Sink Architecture.md`.

The implementation is landing-only. It is not a warehouse serving layer, replay convergence system, or online deduplication system.

## Implementation Scope

The ClickHouse path includes:

1. A generic ClickHouse runtime inside `carbon-core`.
2. Processor finalization so buffered processors drain on shutdown.
3. Typed instruction and CPI-event ClickHouse coverage for `jupiter-swap-decoder`.
4. Typed account ClickHouse coverage for `token-program-decoder`.
5. Renderer support for generated typed account, instruction, and CPI-event landing rows.
6. A per-buffer ClickHouse writer for independent Carbon processes writing into the same table families.
7. Explicit synchronous and async-wait insert settings, with synchronous inserts as the default.
8. Runtime controls for byte-aware batching, global backpressure, transport timeouts, gzip, retries, and optional exact-batch insert deduplication tokens.
9. Renderer-controlled landing-table DDL modes for `MergeTree`, `ReplicatedMergeTree`, and `Distributed` deployments.

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
- `admin.rs` - explicit schema/bootstrap execution
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

- `ClickHouseSchema::operations(config) -> Vec<String>` for ordered schema operations.
- `ClickHouseAdmin::execute_query(...)` for a single query.
- `ClickHouseAdmin::execute_queries(...)` for explicit ordered query lists.
- `ClickHouseAdmin::execute_schema::<S>()` for schema bundles that implement `ClickHouseSchema`.

Schema execution uses the same `ClickHouseConfig` endpoint, database, auth, and HTTP client settings as data inserts. Schema queries include `date_time_input_format=best_effort`.

## What Lives In Decoder Crates

Decoder crates own concrete ClickHouse schemas and row mapping. The current committed generated output is canary-limited.

Jupiter swap instruction and CPI/event canary modules:

- `decoders/jupiter-swap-decoder/src/instructions/clickhouse/mod.rs`
- `decoders/jupiter-swap-decoder/src/instructions/clickhouse/instruction_rows.rs`
- `decoders/jupiter-swap-decoder/src/instructions/clickhouse/cpi_event_row.rs`

Token Program account canary modules:

- `decoders/token-program-decoder/src/accounts/clickhouse/mod.rs`
- `decoders/token-program-decoder/src/accounts/clickhouse/mint_row.rs`
- `decoders/token-program-decoder/src/accounts/clickhouse/multisig_row.rs`
- `decoders/token-program-decoder/src/accounts/clickhouse/token_row.rs`

Those modules provide:

- typed row structs
- table names and DDL
- decoder-family wrapper types
- `ClickHouseRows` implementations
- setup helpers used by examples

The committed Jupiter and Token Program canary modules bootstrap tables with their row types' `create_table_sql(...)` operations. The renderer templates generate the newer `migration_operations(...)` helper that includes additive `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` operations for regenerated decoders.

Examples should call these generated helpers. They should not duplicate decoder-specific schema or row mapping logic.

Generated setup helpers use decoder defaults:

- `clickhouse_config_from_database_url(...)` builds a `ClickHouseConfig` from `DATABASE_URL`.
- `bootstrap_clickhouse_from_database_url(...)` executes the decoder's generated schema operations and returns the config.
- `clickhouse_processor(config)` constructs the generated processor alias.
- `setup_clickhouse(...)` bootstraps schema and returns the generated processor in one call.

Callers that need production settings should use `bootstrap_clickhouse_from_database_url(...)`, then apply `ClickHouseConfig` builders before constructing the processor.

`DATABASE_URL` provides the HTTP endpoint and optional auth credentials. Generated helpers pass their own database default, currently `DEFAULT_DATABASE = "default"`, instead of deriving the database from the URL path.

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
carbon-clickhouse-{table}-{sha256(table + body + attempt)}
```

The query ID is for ClickHouse query-log traceability only. It is not used for deduplication.

When `ClickHouseDeduplicationSettings::ExactBatchHash` is enabled, each exact batch includes:

```text
insert_deduplication_token = sha256(table + "\n" + insert_query + "\n" + exact_body)
```

Retry behavior is controlled by `ClickHouseRetrySettings`. The sink retries network errors, request timeouts, HTTP `408`, `429`, `5xx`, and ClickHouse "too many parts" / "too many inactive parts" responses. Schema errors, auth errors, malformed requests, and most other `4xx` responses are treated as permanent.

## Production Config Examples

Generated decoder setup helpers keep synchronous inserts by default. Production callers can modify the returned `ClickHouseConfig` before constructing the processor.

### Synchronous Backfill

Use synchronous inserts for bounded crawls, deterministic backfills, and replay jobs where the Carbon process should observe insert success before moving on.

```rust
use std::time::Duration;

use carbon_core::clickhouse::{
    ClickHouseBatchSettings,
    ClickHouseDeduplicationSettings,
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
    })
    .with_deduplication_settings(ClickHouseDeduplicationSettings::ExactBatchHash);
```

`ExactBatchHash` emits an `insert_deduplication_token` derived from the exact table, insert query, and body. It is optional because ClickHouse gives explicit tokens priority over the data hash; callers must not reuse a token for different data.

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
- Token Program typed account landing rows for mint, token, and multisig.
- Renderer templates for typed account, instruction, and CPI-event ClickHouse generation.
- `withClickHouse` feature-gated generated modules.
- `withClickHouse: true` for default `MergeTree` DDL.
- `withClickHouse: { ... }` for renderer-controlled DDL options.

The table model is typed landing tables with Postgres-aligned CPI/event grouping:
one table per instruction family, one table per account family, and one
CPI/event table per generated decoder. The CPI/event table uses a discriminator
plus event-prefixed typed payload columns instead of a generic JSON payload.

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
- CPI/event rows: `{program}_cpi_event_landing`
- account rows: `{program}_{account}_account_landing`

Current canary table families include:

- Jupiter instruction tables such as `jupiter_swap_route_instruction_landing`.
- Jupiter CPI/event table: `jupiter_swap_cpi_event_landing`.
- Token Program account tables: `token_program_mint_account_landing`, `token_program_multisig_account_landing`, and `token_program_token_account_landing`.

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

Renderer-generated migrations create tables and then emit additive `ALTER TABLE ... ADD COLUMN IF NOT EXISTS` operations for generated columns. The committed Jupiter and Token Program canaries predate that helper shape and currently bootstrap with `create_table_sql(...)` statements only. The renderer does not generate destructive type-change migrations.

For `distributed` DDL mode, generated migration operations create and alter the local table first, then create and alter the distributed table. The distributed table omits local `MergeTree` clauses because storage lives in the generated local table. Additive column operations are emitted for both local and distributed tables.

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

The TypeScript renderer API supports the object form above. The current CLI flag is still boolean-only: `--with-clickhouse <boolean>`.

## Jupiter Example

`examples/jupiter-swap-clickhouse` is the real-world smoke test for the Jupiter instruction and CPI-event path.

It validates:

- `RpcBlockCrawler` datasource
- production RPC URL loading through `.env`
- ClickHouse bootstrap through generated Jupiter migration helpers
- Jupiter instruction and CPI-event decoding
- typed ClickHouse row dispatch
- structured CPI-event payloads such as `route_plan.swap`
- core writer batching and shutdown drain
- `ShutdownStrategy::Immediate` with a bounded block range
- `BLOCK_CRAWLER_MAX_CONCURRENT_REQUESTS` and `BLOCK_CRAWLER_CHANNEL_BUFFER_SIZE` for local RPC pressure control

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
- `ShutdownStrategy::ProcessPending` for finite account snapshots
- RPC `getProgramAccounts` with token-account data-size/memcmp filters
- Helius gPA v2 through `--source helius-gpa-v2`
- optional `CLICKHOUSE_ASYNC_INSERT*` environment variables

The example requires a token-account owner and/or mint filter so it does not accidentally fetch the entire Token Program account set. By default it focuses on token accounts because those can be bounded safely. The generated mint and multisig account rows are covered by compile/tests and schema generation, but they are not the default real-world query path.

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
- typed decoder-level CPI/event landing row templates with event-prefixed union columns
- generated account `clickhouse/mod.rs`
- generated instruction `clickhouse/mod.rs`
- generated Cargo dependencies for ClickHouse, `serde`, and `chrono`
- strict-by-default ClickHouse schema mapping with explicit `allowClickHouseJsonFallback`
- renderer-controlled DDL planning in `packages/renderer/src/clickhouseDdl.ts`

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
- ClickHouse table design is typed landing tables: one per instruction family, one per account family, and one CPI/event table per generated decoder.

## Responsibility Split

The sink handles:

- typed row conversion from decoder-owned account, instruction, and CPI-event schemas
- deterministic landing IDs
- local row/byte batching and per-buffer flushing
- shutdown drain through processor finalization
- sync insert defaults and async-wait insert settings
- transient HTTP retry/backoff and failed-buffer preservation
- optional exact-batch insert deduplication tokens
- basic generated schema bootstrap for landing tables
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
- Token Program validates account ClickHouse rows.
- Other decoders stay without committed ClickHouse modules until upstream v1 stabilizes.

Broad decoder validation is done without committing generated output. Use `scripts/validate-clickhouse-decoder-rollout.sh` for current canary validation, `--compile-all` for a broader baseline compile scan, `--regenerate-idl-dir` for local IDL files, and `--regenerate-from-readme --rpc-url "$RPC_URL"` for README-listed program IDs. When upstream v1 stabilizes, run broader `withClickHouse` regeneration in a temporary branch or worktree, compile with `--allow-broad-clickhouse`, and commit only the intended rollout set.
