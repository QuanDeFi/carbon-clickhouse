# Carbon Core v1 Architecture

This document summarizes Carbon core v1 as it exists on `upstream-v1-sync`, without the ClickHouse sink branch additions.

Diff basis for this scan:

- Scan timestamp: `2026-05-16 09:05:10 WIB (+0700)`
- Branch scanned: `upstream-v1-sync`
- Commit: `972028f150ef6b83581a21ddb262c8ed75c27bcf`
- Scope: `crates/core`, renderer/CLI integration points, datasource behavior, examples, metrics crates, and generated decoder shape where they clarify core behavior

Branch-state observations in this document are time-sensitive. Recompute them before relying on this document for future upstream sync or merge-risk decisions.

## Workspace And Feature Topology

The scanned upstream workspace includes core crates, metrics crates, datasource crates, generated decoder crates, and examples:

- `crates/*`
- `metrics/*`
- `datasources/*`
- `decoders/*`
- `examples/*`

The workspace package version is `0.12.0`, Rust version is `1.82`, and the upstream dependency set is on Solana 3.x packages.

`carbon-core` default features enable macros only:

- `default = ["macros"]`
- `postgres` is optional and pulls in `sqlx`, `sqlx_migrator`, numeric wrappers, and Postgres primitives.
- `graphql` is optional and pulls in Juniper/Axum support.

There is no upstream `clickhouse` feature.

The upstream CI shape is:

- Rust workflow runs `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo sort -c -g`, `cargo machete`, and `cargo test --all-targets --all-features`.
- The TypeScript workflow uses pnpm `9.6.0`, Node `18`, `pnpm build`, `pnpm format:check`, and `pnpm type-check`.

The TypeScript `packages/versions` package is a central dependency-version registry for generated projects, but it is not a complete mirror of every workspace crate in the scanned commit. For example, some newer datasource crates are present in the workspace but absent from the version registry.

There are no committed TypeScript unit test files in the scanned renderer/CLI packages. The renderer package's `test` script prints `No tests configured`, so upstream TypeScript validation is primarily build, formatting, and type-checking.

## Core Shape

Carbon core v1 is a pipeline framework, not a storage engine.

The runtime flow is:

```text
Datasource
  -> bounded update channel
  -> Pipeline::process
  -> filters
  -> decoder pipe
  -> Processor
```

The central abstractions are:

- `Datasource` emits updates into a channel.
- `Pipeline` owns datasources, processor pipes, metrics exporters, cancellation, and shutdown strategy.
- `Filter` decides whether an update reaches a pipe.
- `AccountDecoder` and `InstructionDecoder` decode raw Solana data into program-owned types.
- `Processor<T>` performs side effects for decoded or raw processor inputs.

`Processor<T>` receives borrowed input:

```rust
fn process(&mut self, data: &T) -> impl Future<Output = CarbonResult<()>> + Send;
```

Upstream v1 has no `Processor::finalize()` lifecycle hook. Buffered sinks need that hook if processed rows can remain in memory after `process()` returns.

## Solana Design Context

Carbon's core shape follows Solana's program model rather than a generic database model.

Program-owned account layouts, instruction layouts, CPI/event layouts, and shared type definitions are the natural schema boundaries. This is why generated decoder crates own account, instruction, event, and type modules, while Carbon core owns the generic pipeline and processor surfaces.

IDL and Codama-driven generation are the intended source of truth for known program schemas. The useful Solana rule is to avoid hand-maintaining Borsh layouts when an IDL/codegen path exists. For ClickHouse-style generated sinks, that means structured row mappings should be derived from decoder/Codama schema, with handwritten canary mappings treated as temporary compatibility scaffolding.

Live RPC, Geyser, account, transaction, and log data should be treated as untrusted input. Account owner checks, data length checks, discriminators, instruction discriminators, and account arrangement validation are not incidental details; they are the safety boundary between arbitrary on-chain bytes and typed decoder output. A decoder that cannot validate a shape should skip or fail that shape rather than silently accepting it as typed data.

CPI/events being modeled as generated instruction variants also follows Solana indexing practice. Program logs can be truncated, while production indexer events can be emitted through CPI instruction data on the program itself. That is why Carbon's Anchor-style `CpiEvent` path is instruction-like and why typed CPI/event landing rows are a decoder-owned family, not a separate core processor surface.

## Datasource Update Model

`Datasource::consume()` receives:

- a `DatasourceId`
- an update sender
- a `CancellationToken`

It sends `(Update, DatasourceId)` into the pipeline channel.

The core update enum has four datasource update types:

```rust
pub enum Update {
    Account(AccountUpdate),
    Transaction(Box<TransactionUpdate>),
    AccountDeletion(AccountDeletion),
    BlockDetails(BlockDetails),
}
```

`TransactionUpdate` carries the signature, full transaction, status metadata, vote flag, slot, transaction index, block time, and block hash.

`Datasource::update_types()` advertises the update families a datasource produces. In upstream v1 this is metadata on the datasource trait; the pipeline does not validate updates against it before processing.

Deep-scan caveat: some datasource implementations emit block details even though their `update_types()` implementation only advertises transactions. `jetstreamer-datasource` and `rpc-block-subscribe-datasource` both emit `Update::BlockDetails`, but their advertised update types are transaction-only in the scanned commit.

`DatasourceDisconnection` is a side-channel struct used by selected streaming datasources to report reconnect gaps through an optional notifier channel. It is not part of the core `Update` enum and is not a processor surface.

`Error::MissingUpdateTypeInDatasource` exists in core, but no scanned core path uses it to enforce `update_types()` consistency.

## Processor Surfaces

The pipeline exposes five processor surfaces:

- accounts
- instructions
- transactions
- account deletions
- block details

There are four datasource update variants, but five processor surfaces because transaction updates are processed in two ways:

- instruction processors receive decoded instructions extracted from the transaction
- transaction processors receive transaction-level metadata plus parsed instructions

CPI/events are not a separate Carbon core processor surface. They are decoder-owned instruction variants. For Anchor-style events, generated decoder code can model events as an instruction variant such as `CpiEvent`.

Generated `types` modules are also decoder-owned support modules. They are not processor surfaces by themselves; they are payload/account/instruction helper types used by account, instruction, event, Postgres, and GraphQL generated code.

## Datasource Ecosystem

Upstream v1 includes multiple datasource crates that all adapt their provider-specific streams or RPC calls into the same core `Update` enum.

Account-producing datasources include:

- RPC GPA
- RPC program subscribe
- Helius GPA v2
- validator snapshot
- Yellowstone gRPC
- Helius LaserStream
- Helius Atlas WebSocket
- stream-message datasource

Transaction-producing datasources include:

- RPC block crawler
- RPC block subscribe
- RPC transaction crawler
- Yellowstone gRPC
- Helius LaserStream
- Helius Atlas WebSocket
- Helius GTFA
- Jito Shredstream gRPC
- Jetstreamer
- stream-message datasource

Account deletion updates are produced by stream/geyser-style datasources such as Yellowstone gRPC, Helius LaserStream, Helius Atlas WebSocket, and stream-message datasource.

Block details updates exist in core and are emitted by block-oriented datasources, but they are not used by the upstream Postgres persistence layer. The scanned `Jetstreamer` and `rpc-block-subscribe` datasources advertise only `Transaction` in `update_types()`, while also emitting `Update::BlockDetails` when block output is enabled.

Some datasources use blocking `send(...).await`; others use `try_send(...)` and can drop or error on backpressure depending on implementation. This matters when reasoning about the central channel buffer of `1_000`.

Observed datasource delivery modes in the scanned commit:

- blocking/awaiting send paths include RPC GPA, Helius GPA v2, Helius GTFA, stream-message, and validator snapshot.
- nonblocking `try_send` paths include Yellowstone gRPC, Helius LaserStream, Jito Shredstream, RPC block crawler, and several streaming paths.
- RPC transaction crawler has a configurable `blocking_send` mode and defaults to nonblocking send.

Datasource matrix from the targeted scan:

| Datasource | Advertised update types | Delivery mode | Notable behavior |
| --- | --- | --- | --- |
| Helius Atlas WebSocket | transaction, account, deletion | `try_send` | tracks account deletions through a configured deletion set |
| Helius GPA v2 | account | awaited `send` | paged account fetch source |
| Helius GTFA | transaction | awaited `send` | paged transaction fetch source |
| Helius LaserStream | account, transaction, deletion | `try_send` | stream source with deletion tracking |
| Jetstreamer | transaction | awaited `send` | can emit block details despite transaction-only metadata |
| Jito Shredstream gRPC | transaction | `try_send` | entry/shredstream transaction source |
| RPC block crawler | transaction | `try_send` | block fetcher emits transaction updates only |
| RPC block subscribe | transaction | `try_send` | can emit block details despite transaction-only metadata |
| RPC GPA | account | awaited `send` | finite account fetch source |
| RPC program subscribe | account | `try_send` | WebSocket account stream source |
| RPC transaction crawler | transaction | configurable, default `try_send` | signature crawler has opt-in blocking send |
| stream-message | account, transaction, deletion | awaited `send` | external unified message source |
| validator snapshot | account | `blocking_send` | finite snapshot source |
| Yellowstone gRPC | account, transaction, deletion | `try_send` | stream source with disconnect notifier and deletion tracking |

Datasource crates also register datasource-local metrics directly into the same global registry. These include fetch/page counters, block/transaction/account counters, and processing-time histograms for individual sources such as RPC block crawler, transaction crawler, Yellowstone, Helius streams, Jito Shredstream, Jetstreamer, and validator snapshots.

Account deletion semantics are datasource-specific. Geyser-style sources infer deletions from zero-lamport, empty, system-owned account updates and only emit deletion updates for accounts tracked in the datasource's `account_deletions_tracked` set. The snapshot validator example wires an `account_deletions` processor, but the scanned snapshot datasource source emits account updates only; its example README appears stale on this point.

## Datasource Failure And Backpressure Semantics

Datasource error handling is not uniform in upstream v1. The pipeline exposes one bounded channel, but each datasource decides whether a full channel is backpressure, a dropped update, or a reconnect trigger.

Observed channel-full behavior:

- Blocking send paths naturally backpressure the datasource until the pipeline receiver catches up. These include RPC GPA, Helius GPA v2, Helius GTFA, stream-message, internal crawler work channels, and validator snapshot `blocking_send`.
- Nonblocking `try_send` paths can drop an update, log an error, return from the current handler, or break a subscription loop. These include Yellowstone gRPC, Helius LaserStream, Helius Atlas WebSocket, Jito Shredstream, RPC block crawler, RPC block subscribe, RPC program subscribe, and RPC transaction crawler by default.
- RPC transaction crawler is configurable: `ConnectionConfig.blocking_send` defaults to `false`, so its default behavior is nonblocking send. The example variant opts into blocking send.

Observed retry/reconnect behavior:

- Helius Atlas WebSocket has finite reconnection attempts, ping/pong timeout, transaction idle timeout, and cancellation-aware subscription loops. A `try_send` failure in account or transaction handling breaks the subscription loop and leads to reconnect.
- Helius LaserStream has connect and stream timeouts, finite reconnect attempts, retry delay, and optional replay from a tracked slot when replay is enabled. Replay starts from the tracked slot, with a processed-commitment rewind.
- Yellowstone gRPC has stream timeout, finite reconnect attempts, and optional disconnection notifier metadata. It reports gaps but does not replay them by itself.
- RPC block subscribe has finite reconnect attempts and optional disconnection notifier metadata. It can emit block details, but its advertised update type is transaction-only.
- RPC program subscribe reconnects after an inner subscription loop breaks; `try_send` failures break that loop.
- RPC transaction crawler retries signature and transaction fetch RPC calls according to `RetryConfig`; those retries do not apply to downstream pipeline send failure.
- Validator snapshot retries snapshot download and checks cancellation during download and scan.
- Jetstreamer accepts a cancellation token in the trait call but the scanned callback wiring does not actively use it.

Production implication: a sink cannot infer end-to-end delivery guarantees from Carbon core alone. Some datasource paths will slow down under backpressure, while others can drop source updates before the sink sees them. Sink-local retries and shutdown drain only protect rows already accepted by the sink processor; they do not recover updates lost by a nonblocking datasource send.

## Pipeline Execution

The pipeline creates one bounded MPSC channel:

```rust
pub const DEFAULT_CHANNEL_BUFFER_SIZE: usize = 1_000;
```

Datasources run as spawned tasks and send updates into that channel. The pipeline has one receiver loop. Each received update is processed through the relevant pipe collections.

This means:

- pipeline intake can be concurrent across datasources
- processor execution is sequential in the central receiver loop
- a slow processor can block further processing from the central queue
- queue depth is central pipeline pressure, not sink-local buffer pressure
- each received update is cloned before processing for error logging and metadata reuse
- metrics are process-global, not pipeline-local
- pipe order matters because the first processor error aborts processing for the current update

The builder supports:

- `datasource(...)`
- `datasource_with_id(...)`
- `account(...)` and `account_with_filters(...)`
- `instruction(...)` and `instruction_with_filters(...)`
- `transaction(...)` and `transaction_with_filters(...)`
- `account_deletions(...)` and `account_deletions_with_filters(...)`
- `block_details(...)` and `block_details_with_filters(...)`
- `metrics(...)`
- `datasource_cancellation_token(...)`
- `channel_buffer_size(...)`
- `shutdown_strategy(...)`

The startup log reports datasource/exporter/account/account-deletion/instruction/transaction pipe counts, but it does not include the block-details pipe count in the scanned upstream commit.

The metrics registry is append-only and process-global. `PipelineBuilder::build()` registers pipeline metrics each time it is called, and the static counter/gauge/histogram values are shared process-wide. Multiple pipelines in one process therefore need care: metrics values are not isolated by pipeline, and repeated registration can duplicate descriptors in snapshots/export output.

For transaction updates, instruction pipes run before transaction pipes. If any instruction processor returns an error, the `?` propagation exits the update processing path before later instruction pipes, transaction pipes, and the transaction processed counter run.

## Shutdown Model

Upstream v1 has two shutdown strategies:

- `Immediate`
- `ProcessPending`

On cancellation or Ctrl-C, the pipeline exports metrics and shuts down exporters. With `ProcessPending`, Ctrl-C cancels datasources and continues processing pending updates until the update channel closes.

There is no upstream processor finalization step. That is an important limitation for buffered sinks: a processor can have accepted data in `process()` but not yet flushed it externally.

Fourth-scan caveat: the implementation has a `datasource_cancellation_token.cancelled()` select branch that immediately exports metrics, shuts down exporters, and breaks the loop. The Ctrl-C path for `ProcessPending` cancels that same token. In practice, this makes Ctrl-C pending-drain behavior fragile: after Ctrl-C, the cancellation branch can win before the receiver drains all queued updates. Finite datasources can still drain naturally when senders close without external cancellation.

## Pipeline Test Coverage Gaps

The scanned `crates/core/src/pipeline.rs` has no direct unit tests. Existing core tests primarily cover instruction nesting/log-event extraction and transaction/account transformer fixtures.

Important behaviors not directly covered by scanned tests:

- `ShutdownStrategy::Immediate` versus `ShutdownStrategy::ProcessPending`
- Ctrl-C and external cancellation interaction with pending-channel drain
- pipe ordering, especially instruction pipes before transaction pipes
- first processor error aborting later pipes for the same update
- metrics registration behavior when multiple pipelines are built in one process
- queue-depth gauge semantics under full-channel pressure
- block-details pipe count omission from the startup log

These gaps matter for sink work because buffered processors depend on deterministic shutdown/drain behavior, while metrics and queue depth are process-global and not sink-local.

## Account Path

Account updates are converted into `AccountMetadata`:

- slot
- pubkey
- optional transaction signature

The account pipe:

1. applies account filters
2. calls the decoder's `decode_account(...)`
3. builds `AccountProcessorInputType`
4. calls the processor

`AccountProcessorInputType` includes:

- `metadata`
- `decoded_account`
- `raw_account`

## Instruction Path

Transaction updates are transformed into flat `InstructionsWithMetadata`, then converted into `NestedInstructions`.

Instruction metadata includes:

- transaction metadata
- stack height
- outer instruction index
- absolute path

The instruction pipe:

1. flattens nested instructions
2. applies instruction filters per nested instruction
3. calls the decoder's `decode_instruction(...)`
4. builds `InstructionProcessorInputType`
5. calls the processor

`InstructionProcessorInputType` includes:

- `metadata`
- `decoded_instruction`
- `nested_instructions`
- `raw_instruction`

Instruction metadata also supports event log extraction through `decode_log_events<T>()`. This reads transaction log messages and matches logs by adjusted instruction path.

Generated instruction decoding uses `try_decode_instructions!`, which attempts each generated instruction type in order. A variant is accepted only if both payload decode and account arrangement succeed. Account arrangement uses `next_account(...)`; missing required accounts reject the variant, while missing optional accounts become `None`.

Transaction-to-instruction extraction has a few data-quality fallbacks:

- missing program IDs become the default public key
- invalid loaded-address strings become default public keys
- unsupported parsed UI instructions are converted into empty compiled instructions
- invalid base58 UI instruction data becomes an empty data vector

Those fallbacks keep ingestion moving but can produce undecodable or degraded instructions instead of hard failures.

## CPI/Event Decoder Shape

CPI/events are modeled as generated instruction variants. In the scanned upstream v1 branch:

- 39 decoder crates contain `src/instructions/cpi_event.rs`
- those decoders contain 446 generated event payload structs under `src/events`
- `CpiEvent` uses the Anchor event instruction discriminator first, then each payload struct checks its own event discriminator
- generated CPI-event account arrangement accepts either `[program, event_authority, remaining @ ..]` or `[event_authority]`

Largest CPI/event decoders by event variant count:

| Decoder | CPI/event variants |
| --- | ---: |
| marginfi-v2-decoder | 30 |
| drift-v2-decoder | 26 |
| boop-decoder | 25 |
| jupiter-perpetuals-decoder | 25 |
| marinade-finance-decoder | 25 |
| meteora-dlmm-decoder | 25 |
| meteora-damm-v2-decoder | 23 |
| meteora-dbc-decoder | 23 |
| pumpfun-decoder | 23 |
| circle-token-messenger-v2-decoder | 22 |
| pump-swap-decoder | 22 |

Event payload field shapes from the targeted scan:

| Shape | Field count | Event files | Decoder count |
| --- | ---: | ---: | ---: |
| total event payload fields | 2,780 | 446 | 39 |
| `Pubkey` fields | 979 | 385 | 37 |
| `u128` / `i128` fields | 150 | 65 | 13 |
| `Option<...>` fields | 81 | 26 | 8 |
| `Vec<...>` fields | 19 | 18 | 10 |
| fixed array fields | 16 | 16 | 8 |

Notable event schema stress cases:

- Drift v2 has many `u128`/`i128` event fields, optional scalar/pubkey fields, nested defined-type payloads, and events such as `OrderActionRecord` and `LiquidationRecord` with many fields.
- Jupiter Swap has only six event variants, but `SwapsEvent` and `CandidateSwapResults` use vectors of defined types, which is why it is a useful ClickHouse event canary.
- Marginfi v2 uses many nested header/config defined types and an `Option<Vec<Pubkey>>` event field.
- Marinade Finance uses many `Option<DefinedType>` configuration-change fields.
- Circle Message Transmitter and Drift include fixed byte arrays in event payloads.

Upstream generated Postgres persistence for CPI/events is intentionally generic: every event variant goes into one `cpi_events` table with `name TEXT`, `data JSONB`, and `__accounts JSONB`. Broad typed ClickHouse event rollout is therefore not a one-to-one copy of upstream Postgres. It requires structured mapping coverage for all known event payload shapes or an explicit fail-fast path.

## Transaction Path

Transaction processors are separate from instruction processors.

The transaction path builds `TransactionMetadata` from `TransactionUpdate`, then parses instructions through an `InstructionDecoderCollection`.

`TransactionProcessorInputType` includes:

- transaction metadata
- decoded instruction list

This surface is useful for transaction-level aggregation. It is not how upstream Postgres persistence is implemented by default.

Important caveat: `TransactionUpdate` has `is_vote`, but upstream v1 `TransactionMetadata` does not preserve `is_vote`. Transaction processors that only receive `TransactionMetadata` cannot see the datasource vote flag without core changes.

`InstructionDecoderCollection` requires decoded instruction collection values to be `Clone + Debug + Send + Sync + Eq + Hash + Serialize + 'static`. The versioned-decoder example avoids `InstructionDecoderCollection` and instead registers two `.instruction()` pipes with `SlotRangeFilter`, because generated instruction enums do not always satisfy the stronger collection bounds.

## Account Deletion And Block Details Paths

Account deletion and block details updates are raw processor surfaces.

They do not use decoder-owned schema in core v1:

- `AccountDeletionPipe<P>` passes `AccountDeletion` to a processor.
- `BlockDetailsPipe<P>` passes `BlockDetails` to a processor.

Both support filters.

## Filters

Filters can be applied to every processor surface:

- account
- instruction
- transaction
- account deletion
- block details

Each filter method defaults to `Accept`.

Built-in filters include:

- `DatasourceFilter`
- `SlotRangeFilter`
- `DeduplicationFilter`

`DatasourceFilter` accepts updates from selected `DatasourceId`s.

`SlotRangeFilter` gates updates by slot and, for transaction/instruction surfaces, optional transaction index boundaries.

`DeduplicationFilter` only deduplicates instructions and accounts:

- instructions: `(signature, absolute_path)`
- accounts: `(transaction_signature, pubkey)`

It is an in-memory TTL filter, not durable deduplication.

`SlotRangeFilter` is inclusive at the lower slot boundary and exclusive at the upper transaction-index boundary when an upper transaction index is supplied. This is why the versioned-decoder example can split an upgrade at `(slot, tx_index)` without overlapping both decoders on the upgrade transaction.

## Metrics

Core metrics are static, global, and label-less.

Pipeline counters:

- `carbon_updates_received_total`
- `carbon_updates_processed_total`
- `carbon_updates_successful_total`
- `carbon_updates_failed_total`
- `carbon_account_updates_processed_total`
- `carbon_transaction_updates_processed_total`
- `carbon_account_deletions_processed_total`
- `carbon_block_details_processed_total`

Pipeline gauge:

- `carbon_updates_queued`

Pipeline histograms:

- `carbon_updates_process_time_nanoseconds`
- `carbon_updates_process_time_milliseconds`

`carbon_updates_queued` is the central update channel depth. It is not sink-local buffer occupancy.

Metrics exporters are initialized before datasource tasks start, receive snapshots during pipeline execution and shutdown, and expose a `shutdown()` hook.

The upstream metrics exporter crates are outside `crates/core` but implement the core `MetricsExporter` trait:

- `carbon-log-metrics` periodically logs snapshots from a background task and cancels that task on exporter shutdown.
- `carbon-prometheus-metrics` can expose `/metrics` on `0.0.0.0:9464` when built with the HTTP server feature.

The upstream Prometheus formatter sanitizes hyphens in metric names. Upstream core and Postgres metric names use underscores, so this is sufficient for upstream v1 as scanned. Metric names containing dots require additional handling outside upstream v1.

Because metrics are label-less, upstream v1 models separation through metric names rather than labels. There is no native pipeline ID, datasource ID, decoder name, table name, or process role label in the core registry.

## Built-In Postgres Support

Postgres is an optional core feature.

Core Postgres support covers accounts and instructions:

- `PostgresAccountProcessor`
- `PostgresJsonAccountProcessor`
- `PostgresInstructionProcessor`
- `PostgresJsonInstructionProcessor`

Generic JSON tables:

- `accounts`
- `instructions`

Generic account rows use `__pubkey` as the primary key and store decoded data as JSONB.

Generic instruction rows use `(__signature, __instruction_index, __stack_height)` as the primary key and store decoded instruction data as JSONB.

Generated typed decoder Postgres rows live in decoder crates. Core provides traits and processors; decoder crates own schema-specific row wrappers.

Postgres metrics are account/instruction upsert counters and duration histograms. Upstream v1 does not provide generic Postgres processors for transactions, account deletions, or block details.

The renderer's typed Postgres mapper flattens straightforward fields to SQL columns, but it intentionally stores many complex shapes as `JSONB`, including defined types, maps, sets, non-singleton tuples, and arrays of complex items. This is why upstream Postgres can tolerate many schema shapes without requiring a fully structured SQL representation for every nested type.

Postgres operation traits include `Insert`, `Upsert`, `Delete`, and `Lookup`; generated typed rows and generic rows implement these operations. The provided processors use `Upsert`.

Important persistence caveat: neither generic nor generated Postgres instruction rows include `absolute_path` in the primary key. The key is signature, outer instruction index, and stack height. Multiple sibling inner instructions can share those three values and differ only by absolute path, so upstream Postgres can overwrite/collide in those cases.

Upstream generated CPI/event Postgres support is a single `cpi_events` table with `name TEXT` and `data JSONB`, not one typed table per event family.

Generated Postgres migration operations are create/drop style operations around `CREATE TABLE IF NOT EXISTS` and `DROP TABLE IF EXISTS`. The scanned generated Postgres output does not include additive `ALTER TABLE ... ADD COLUMN` migration operations, so schema evolution is not handled incrementally by upstream generated persistence.

## Built-In GraphQL Support

GraphQL is an optional core feature.

Core GraphQL support provides:

- `build_schema(...)`
- `graphql_router(...)`
- scalar wrappers for Solana public keys, large integers, and JSON values

Generated GraphQL schema/query modules live in decoder crates.

GraphQL scalar wrappers stringify large integer values to preserve precision. The generic JSON scalar renders large integer JSON numbers as strings on output and accepts JSON input as a string.

## Renderer And CLI Direction

Upstream v1 renderer and CLI support:

- `withPostgres`
- `withGraphQL`
- `postgresMode: generic | typed`
- generated account modules
- generated instruction modules
- generated event modules
- generated type modules
- generated Postgres modules
- generated GraphQL modules

CLI defaults matter for regeneration: the CLI defaults `withPostgres` and `withGraphQL` to true, and defaults `withSerde` to true only when both are disabled. Renderer code generally treats Postgres and GraphQL as enabled unless the option is explicitly false.

The generated decoder crate shape in the scanned workspace is:

- 63 decoder crates
- all 63 have instruction variants
- 56 have concrete account variant files
- 39 have generated Anchor/CPI event modules
- 56 have generated type modules
- no decoder has a generated transaction module

Across those decoders, the scanned tree contains 1,827 instruction variant files, 295 account variant files, 446 event files, and 1,547 type files. These numbers reinforce that upstream v1's generated, decoder-owned data families are accounts, instructions, events, and types. Transactions remain a core processor surface, not a generated decoder data family.

Seven committed decoder crates have no concrete account variant files: associated-token-account, memo-program, okx-dex, phoenix-v1, raydium-stable-swap, stake-program, and swig. All committed decoder crates have `serde`, `postgres`, and `graphql` feature sections. None has a committed `base58` feature, and 26 use `serde-big-array`.

High-complexity decoder stress cases from the targeted scan:

| Decoder | Account variants | Instruction variants | CPI/event variants | Type variants | Postgres files | GraphQL files | Stress shape |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| drift-v2-decoder | 27 | 248 | 26 | 139 | 277 | 420 | many large fixed arrays, nested account structs, complex enums, and many Postgres `JSONB` payload fields |
| token-2022-decoder | 3 | 91 | 0 | 8 | 96 | 108 | SPL Token 2022 extension account decoding and optional extension vectors |
| meteora-dlmm-decoder | 12 | 75 | 25 | 65 | 89 | 158 | CPI events plus liquidity parameter structs and repeated complex instruction payloads |
| raydium-clmm-decoder | 9 | 26 | 11 | 16 | 37 | 57 | nested tick/observation arrays, fixed arrays, and CPI events |

These decoders explain why a strict typed sink needs broad non-committing validation before regenerating everything. Upstream Postgres uses JSONB as a pressure valve for known complex shapes; a structured ClickHouse mapper must either support those shapes explicitly or fail fast instead of silently producing an unusable schema.

Renderer schema-edge scan across committed decoders found these high-risk shapes outside generated Postgres and GraphQL output:

| Shape | Count | Files | Decoder count | Notes |
| --- | ---: | ---: | ---: | --- |
| `u128` / `i128` fields | 762 | 343 | 29 | Requires precision-preserving mapping; Drift v2 dominates this class. |
| `Option<...>` fields | 1,286 | 513 | 44 | Includes optional composites that Postgres often routes through JSONB. |
| `Option<Vec<...>>` fields | 30 | 25 | 6 | Important because ClickHouse does not allow `Nullable(Array(...))`. |
| `Option<Composite>` fields | 328 | 168 | 26 | Requires explicit presence handling or fail-fast mapping. |
| `Vec<...>` fields | 2,015 | 1,938 | 62 | Common across instructions, accounts, events, and types. |
| `Vec<Composite>` fields | 134 | 124 | 34 | Jupiter Swap route plans are the canary version of this shape. |
| `Vec<Vec<...>>` fields | 3 | 3 | 3 | Rare but structurally important for strict typed sinks. |
| fixed array fields | 792 | 394 | 39 | Many use `serde_big_array::BigArray`; fixed byte arrays are common. |
| `HashMap` / `HashSet` fields | 4 | 2 | 2 | Postgres maps/sets fall back to JSONB upstream. |
| enum declarations | 457 | 457 | 63 | Approximate scan found 77 payload-like enums and 317 fieldless-like enums in direct instruction/type modules. |

Top stress decoders by selected shape:

- `Vec<Composite>`: pump-fees, Jupiter Swap, Jupiter Lend, Drift v2, Meteora DLMM, MPL Core, Boop, OpenBook v2, DFlow Aggregator v4, Pumpfun.
- `Option<Composite>`: MPL Core, Marinade Finance, Token 2022, Marginfi v2, Drift v2, MPL Token Metadata, Jupiter Perpetuals, Orca Whirlpool, Token Program, Bubblegum.
- fixed arrays: Marginfi v2, Zeta, Drift v2, Kamino Lending, Meteora DLMM, Wavebreak, Meteora DAMM v2, OpenBook v2, Kamino Vault, Bubblegum.
- large integers: Drift v2, Meteora DAMM v2, Raydium CLMM, Pancake Swap, Orca Whirlpool, Jupiter Lend, Kamino Lending, Meteora DBC, Wavebreak, Meteora DLMM.

Renderer output deletes the target directory by default. CLI `--no-clean` disables that behavior.

Anchor IDL generation applies several normalization steps before rendering:

- legacy Anchor events are converted into defined types plus modern event discriminators
- selected IDL v0.1 PDA seed paths are normalized
- problematic PDA account seed references can be removed
- nested instruction arguments are preserved when Codama's default flattening would inline them

Codama IDL generation follows a narrower path and applies `extractStructArrayItems()` so anonymous struct array elements become named defined types. This avoids Rust shapes like `Vec<{ anonymous struct }>`.

Token-2022 generation is special-cased. Generated account decoding uses SPL Token 2022 `StateWithExtensions` for mint/token accounts, handles multisig separately, and emits extra conversion code for extension types.

There is no ClickHouse support in upstream v1:

- no `clickhouse` feature in `carbon-core`
- no `carbon_core::clickhouse`
- no `withClickHouse`
- no ClickHouse renderer templates
- no ClickHouse DDL planner
- no ClickHouse example crates

The generator stack also includes macro support:

- `try_decode_instructions!` tries each generated instruction decoder and account arranger in order.
- `instruction_decoder_collection!` builds multi-program transaction decoder collections with optional direct program-id dispatch and a fallback decode path.
- `#[derive(InstructionType)]` generates an enum of instruction kinds for a generated instruction enum.

Deep-scan caveat: the CLI scaffold template's non-Postgres branch still references an older processor API shape (`MetricsCollection`, associated `InputType`, and `impl Processor for ...`) that does not match upstream v1's current borrowed-input `Processor<T>` trait. Postgres scaffold paths use the current processor types. Treat non-Postgres scaffold output as suspect unless verified against the current branch.

The root README also contains older snippets that describe datasources as account/transaction/account-deletion only and show the pre-v1 processor API with `MetricsCollection`. The implementation in `crates/core/src` is the source of truth for this scan.

TypeScript CI uses Node 18 with pnpm 9.6 and runs build, format-check, and type-check. The renderer package's `test` script prints `No tests configured`, so renderer behavior changes need branch-local tests or explicit generated-output checks if they are expected to be enforced.

## Example Coverage

The upstream examples exercise different parts of the architecture:

| Example | Datasource shape | Processor surfaces | Smoke-test value | Caveat |
| --- | --- | --- | --- | --- |
| `block-subscribe-rpc` | RPC WebSocket block subscribe | instructions, block details | Best upstream no-Geyser example for instruction plus block-details behavior. | Requires an RPC endpoint that supports `blockSubscribe`; source advertises transaction-only metadata despite block-details output. |
| `gpa-rpc` | RPC GPA or Helius GPA v2 | accounts | Best upstream account-surface example. | State snapshot/fetch path, not live deletion or transaction flow. |
| `transaction-crawler-rpc` | RPC signature crawler or Helius GTFA | instructions | Good bounded historical instruction backfill example. | Default crawler can use nonblocking send unless configured; no account or event-specific persistence. |
| `yellowstone-grpc` | Yellowstone, LaserStream, or Jito variant | instructions | Best live streaming instruction/CPI-event candidate when Geyser-style endpoints are available. | CPI/events only appear through decoder instruction variants; not a separate processor surface. |
| `postgres-graphql` | Yellowstone account stream | accounts | Canonical upstream persistence/query example with Postgres JSON accounts plus GraphQL. | Postgres example covers account persistence only; no transaction, deletion, block-details, or typed event persistence. |
| `jetstreamer` | Old Faithful archive stream | instructions | Useful finite archive/backfill example. | Can emit block details internally depending on config, but example only wires instructions. |
| `snapshot-validator` | validator snapshot file/URL | accounts, account deletions in example wiring | Good account snapshot example. | Scanned datasource emits account updates only; account-deletion processor wiring appears stale. |
| `versioned-decoders` | mock datasource | instructions with slot filters | Useful for decoder-version routing and `SlotRangeFilter` semantics. | Not a live datasource smoke test. |
| `custom-datasource` | local custom RPC datasource | instructions | Useful for custom datasource authoring and pipeline shape. | Less useful for production datasource failure/backpressure behavior. |

The examples are useful architecture references, but several README/code snippets are stale relative to the current upstream processor trait and update family set.

## Future Upstream Branch Risk

The branch scan was refreshed with `git fetch upstream --prune` during this document update.

Computed comparison against `upstream/v1.0-rc`:

| Branch | `v1.0-rc`-only commits | Branch-only commits | Latest commit | Merge-risk read |
| --- | ---: | ---: | --- | --- |
| `upstream/refactor/core-consistency` | 0 | 22 | `940ca846f` on 2026-05-04 | High-signal follow-up branch. It is directly ahead of `v1.0-rc`. |
| `upstream/main` | 164 | 2 | `9dc3c3a5b` on 2026-04-30 | Not a v1 follow-up branch. It is mostly behind `v1.0-rc`; unique change is a workspace dependency path fix. |
| `upstream/feature/parallel-datasource-workers` | 1,021 | 1 | `8d50cd5e5` on 2025-02-16 | Old experimental branch; conceptually relevant to pipeline pressure, but far behind v1. |
| `upstream/feat/atomic-metrics` | 339 | 3 | `48ade12a5` on 2025-11-20 | Older metrics branch; not current relative to v1. |
| `upstream/nd/arc-in-pipeline` | 249 | 8 | `940676bb7` on 2025-12-03 | Older pipeline-refactor branch; far behind v1. |
| `upstream/nd/arranged-accounts-refactor` | 249 | 16 | `efe023190` on 2025-12-05 | Older decoder/account-arranging branch; far behind v1. |
| `upstream/nd/reference-refactor` | 201 | 16 | `58eb3c874` on 2026-01-12 | Older reference/refactor branch; far behind v1. |

`upstream/refactor/core-consistency` was merged into `clickhouse-upstream-v1`
during the 2026-05-18 WIB maintenance pass. The branch touched the areas most
likely to conflict with local ClickHouse additions:

- Core pipe structs get private fields plus `new(...)` constructors.
- `Filter` object bounds drop redundant `Send + Sync` annotations in several builder and pipe signatures.
- `Datasource` dyn types similarly drop redundant `Send + Sync` object annotations.
- `solana-program` is removed from core in favor of narrower Solana crates such as `solana_hash`.
- Metrics lock poisoning paths use `.expect(...)` instead of `unwrap()`.
- `postgres::operations::LookUp` was renamed to `Lookup`, requiring generated Postgres template and generated decoder updates.
- The branch reformats a large number of generated Postgres/GraphQL files due to the trait rename and formatting changes.

The `refactor/core-consistency` diff over scanned core/renderer/datasource/decoder/example/metrics paths is 2,261 files, 3,261 insertions, and 2,735 deletions. Most changes are comments, formatting, constructor encapsulation, and the Postgres trait rename rather than a new pipeline abstraction. For the ClickHouse branch, the practical merge-risk areas are `pipeline.rs`, pipe structs, metrics registry, Postgres operation names in templates, and any generated ClickHouse code that mirrors upstream pipe-constructor patterns.

### Merge Scan And Resolution Against ClickHouse Branch

A dry merge was run in a disposable worktree at `2026-05-16 09:11:05 WIB (+0700)`:

- Merge target: `clickhouse-upstream-v1` at `6b84bec8f`
- Merge source: `upstream/refactor/core-consistency` at `940ca846f`
- Merge base: `972028f15`
- Result: merge conflicts, with no direct conflict in ClickHouse-specific files

Unmerged files from the dry merge:

| File | Conflict cause |
| --- | --- |
| `README.md` | documentation drift between the ClickHouse branch and upstream core-consistency docs |
| `crates/core/src/account.rs` | ClickHouse branch adds pipe `finalize()`; upstream changes filter object bounds and pipe encapsulation |
| `crates/core/src/account_deletion.rs` | same `finalize()` versus filter-bound/encapsulation conflict |
| `crates/core/src/block_details.rs` | same `finalize()` versus filter-bound/encapsulation conflict |
| `crates/core/src/instruction.rs` | same `finalize()` versus filter-bound/encapsulation conflict |
| `crates/core/src/transaction.rs` | same `finalize()` versus filter-bound/encapsulation conflict |
| `crates/core/src/pipeline.rs` | ClickHouse branch adds `finalize_pipes()`; upstream moves helper methods and refactors pipe construction |

Non-conflicting but relevant auto-merged paths include:

- `crates/core/src/processor.rs`: ClickHouse branch's default `Processor::finalize()` coexists with upstream's documentation changes.
- `crates/core/src/metrics.rs`: upstream poison-lock handling and docs auto-merge, but metric registry remains global and label-less.
- `metrics/prometheus-metrics/src/lib.rs`: upstream import/style changes auto-merge; ClickHouse metric-name handling still needs review during an actual merge.
- `packages/renderer/templates/eventInstructionRowPage.njk`, `graphqlQueryPage.njk`, and `postgresRowPage.njk`: upstream `LookUp` to `Lookup` rename affects Postgres templates, not ClickHouse templates directly.
- `examples/yellowstone-grpc/src/main.rs`: upstream example edits auto-merge and do not affect committed ClickHouse examples directly.

Practical resolution direction for a future merge:

- Keep the ClickHouse branch's `Processor::finalize()` lifecycle and `Pipeline::finalize_pipes()` shutdown drain.
- Adopt upstream's simplified `Box<dyn Filter + 'static>` object bounds consistently in pipe traits and implementations.
- Keep upstream's private pipe fields and `new(...)` constructors; the dry merge already auto-applies these in `PipelineBuilder`.
- Re-run core ClickHouse tests after resolving the six core conflicts because the conflicts are exactly in the processor lifecycle path used by buffered ClickHouse writers.
- Re-run renderer checks because upstream's Postgres template rename touches generated-output templates even though ClickHouse templates do not conflict directly.

Actual merge resolution on `2026-05-18 01:18:01 WIB (+0700)` used that plan:

- Merge target before merge: `clickhouse-upstream-v1` at `a834ed91f`.
- Merge source: `upstream/refactor/core-consistency` at `940ca846f`.
- Kept `Processor::finalize()` and `Pipeline::finalize_pipes()` for buffered ClickHouse shutdown drain.
- Adopted upstream's simplified `Box<dyn Filter + 'static>` object bounds and pipe constructor encapsulation.
- Resolved the Jupiter GraphQL conflict to upstream's `postgres::operations::Lookup` spelling.
- Kept README ClickHouse CLI option docs while incorporating upstream's refreshed README structure.

## Architectural Implications For Sink Work

New sinks should align with upstream v1 when they:

- stay processor-driven
- keep decoder-owned schema in decoder crates
- use filters through the existing pipe model
- expose metrics through the global registry
- avoid changing core channel and fanout semantics unless explicitly required

Buffered sinks need extra lifecycle support because upstream v1 processors have no finalization. A buffered sink that returns from `process()` before data is durably written needs a shutdown drain path outside upstream v1's original lifecycle.

The risky core areas to modify casually are:

- `pipeline.rs`
- processor trait contracts
- central channel/fanout semantics
- metrics registry shape

Those areas define Carbon's v1 architecture and affect every datasource, decoder, and processor.
