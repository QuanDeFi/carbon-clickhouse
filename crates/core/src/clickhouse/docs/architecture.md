# ClickHouse Sink Architecture

## Purpose

This document explains the ClickHouse sink architecture currently implemented on the upstream-v1 port of Carbon, why it is designed this way, and where it intentionally differs from other Carbon sink integrations.

The current implementation is a minimal but real end-to-end ClickHouse path. It is intentionally narrower than the long-term target architecture.

Current implemented scope:
- instruction/CPI-event ingestion only
- one concrete decoder path: Jupiter swap `CpiEvent::SwapEvent`
- one landing table family: `jupiter_swap_swap_event_landing`
- append-only writes into ClickHouse
- client-side batching
- buffered-write drain on pipeline shutdown

Deferred scope:
- account-family ClickHouse support
- broader per-decoder ClickHouse coverage
- serving/canonicalization tables
- coverage/range tracking
- async inserts as a primary ingest model

## Design Goals

The ClickHouse sink is designed around these requirements:
- fit Carbon's processor-driven architecture instead of introducing a separate sink subsystem
- keep examples thin and push storage-specific logic into decoder-owned modules
- support ClickHouse efficiently without pretending it behaves like Postgres
- preserve deterministic row identity for replayed or repeated ingestion
- keep landing-table ingestion simple and append-only
- flush buffered rows safely on shutdown
- stay compatible with Carbon's code generation model

## Where ClickHouse Fits In Carbon

The ClickHouse integration is not a special pipeline type. It is a normal Carbon processor.

The flow is:

1. a datasource emits transaction updates
2. Carbon transforms those updates into decoded instruction inputs
3. an instruction processor receives borrowed instruction input
4. the ClickHouse processor converts that input into backend-specific row wrappers
5. decoder-owned ClickHouse code maps those wrappers to landing rows
6. the generic ClickHouse writer batches and inserts those rows into ClickHouse

That keeps the integration aligned with Carbon's existing processor model instead of building a parallel sink framework.

## Architectural Layers

### 1. Core ClickHouse Runtime in `carbon-core`

Implemented in:
- `crates/core/src/clickhouse/config.rs`
- `crates/core/src/clickhouse/http.rs`
- `crates/core/src/clickhouse/admin.rs`
- `crates/core/src/clickhouse/rows/mod.rs`
- `crates/core/src/clickhouse/writer.rs`
- `crates/core/src/clickhouse/processors.rs`

This layer owns generic ClickHouse behavior only:
- connection config
- minimal HTTP transport
- schema execution
- row/table traits
- batched buffered writing
- instruction processor integration
- ClickHouse metrics registration

This layer does not know Jupiter-specific schema details.

### 2. Decoder-Owned ClickHouse Integration

Implemented in:
- `decoders/jupiter-swap-decoder/src/instructions/clickhouse/mod.rs`
- `decoders/jupiter-swap-decoder/src/instructions/clickhouse/cpi_event_row.rs`

This layer owns program-specific sink logic:
- the decoder wrapper type used by the generic processor
- row mapping from decoded instruction/event data to ClickHouse rows
- table DDL for the family
- decoder-specific bootstrap helpers
- default ClickHouse runtime settings for that decoder family

This matches how Postgres integration is structured in Carbon: generic backend behavior lives in core, while decoder-specific schema and dispatch live in decoder crates.

### 3. Code Generation Support

Implemented in:
- `packages/renderer/templates/instructionsClickHouseMod.njk`
- `packages/renderer/templates/eventInstructionClickHouseRowPage.njk`
- `packages/renderer/src/getRenderMapVisitor.ts`
- `packages/renderer/src/cargoTomlGenerator.ts`

This layer exists so ClickHouse support can scale through Carbon's generator system rather than becoming a handwritten per-decoder integration forever.

The current generator support is intentionally partial. It covers the minimal instruction/CPI-event pattern needed for this v1 path.

### 4. Thin Example Layer

Implemented in:
- `examples/jupiter-swap-clickhouse/src/main.rs`

The example is intentionally small. It should not contain sink-specific mapping logic or DDL details. It should only:
- load environment
- construct datasource
- call decoder-owned ClickHouse setup
- attach the processor to the pipeline
- run the pipeline

That keeps the example aligned with Carbon's broader integration style.

## Why The Sink Is Processor-Driven

Carbon's architecture is already processor-driven:
- pipelines route decoded data into processors
- processors own side effects
- decoders own the mapping between decoded domain data and storage-specific outputs

The ClickHouse sink follows this model instead of introducing a first-class sink trait or external write framework.

Reasons:
- it keeps the ClickHouse path consistent with the rest of Carbon
- it minimizes conceptual surface area in the pipeline
- it allows ClickHouse support to compose naturally with existing decoder and processor abstractions
- it keeps decoder-owned schema generation possible

## Why ClickHouse Differs From Postgres

From an integration perspective, ClickHouse follows the same high-level Carbon model as Postgres:
- processor-driven execution
- decoder-owned schema and row mapping
- thin examples
- generator-backed decoder integration

But the runtime behavior is intentionally different.

Postgres in Carbon is row-at-a-time.
ClickHouse is batched and buffered.

That difference is not cosmetic. It exists because ClickHouse performs best with fewer, larger inserts, especially for append-oriented analytical workloads. Treating ClickHouse like row-at-a-time Postgres would increase part creation pressure and waste one of ClickHouse's main performance advantages.

So the ClickHouse sink deliberately differs in these ways:
- it buffers rows client-side
- it flushes on size or time thresholds
- it needs explicit shutdown draining
- it treats landing-table inserts as append-only

The goal was not to copy Postgres mechanically. The goal was to preserve Carbon's integration style while using a ClickHouse-appropriate write model.

## Why We Chose Client-Side Batching

The current implementation uses client-side batching in `ClickHouseBatchWriter`.

Implemented behavior:
- rows are grouped by partition key in memory
- the writer flushes when `max_rows` is reached
- the writer also flushes when `flush_interval` is reached
- flush sends `JSONEachRow` batches over HTTP

Reasons this was chosen:
- ClickHouse benefits from larger batched inserts
- Carbon workloads include backfills and bounded crawlers, not just tiny live writes
- client-side batching gives deterministic control over batch contents and order
- it makes landing-table ingestion predictable and easy to reason about
- it fits replay-heavy ingestion better than relying purely on server-side async buffering

We intentionally did not make `async_insert` the primary runtime model.

Why not:
- it shifts batching behavior to the server
- it weakens control over exactly when and how batches are formed
- it is less attractive for a pipeline that already has a natural buffered boundary at the processor/writer layer
- it does not remove the need to think about orderly shutdown and delivery guarantees

Async inserts may still be a later optimization. They are not the primary model of this sink.

## Why We Added Processor Finalization

The buffered ClickHouse sink must drain accepted rows before the pipeline exits.

Because of that, upstream-v1 Carbon was extended so processors can finalize.

Implemented in:
- `crates/core/src/processor.rs`
- `crates/core/src/pipeline.rs`
- all pipe wrappers that delegate processor finalization

Why this was necessary:
- a buffered writer can contain rows that were accepted by the processor but not yet flushed
- without a pipeline-level lifecycle hook, those rows can be lost on shutdown
- timer-based flush alone is not enough
- relying on `Drop` for async I/O would be fragile and incorrect

So the current rule is:
- ClickHouse processors flush buffered rows in `finalize()`
- the pipeline calls `finalize()` before completing shutdown

This is the main architectural core change introduced by the ClickHouse port. It is justified because it solves a real correctness problem for buffered sinks.

## Why We Use a Minimal HTTP Transport

The ClickHouse transport layer is intentionally small.

Implemented in:
- `crates/core/src/clickhouse/http.rs`

It exposes two shared helpers:
- `post_query(...)`
- `post_query_with_data(...)`

This shape was chosen to match ClickHouse's own lightweight HTTP testing style rather than building a heavy client abstraction.

Reasons:
- it keeps transport logic easy to audit
- schema execution and data inserts use the same transport model
- it avoids duplicating authentication and HTTP error handling
- it keeps the sink close to ClickHouse's native HTTP model

Current insert format:
- `JSONEachRow`

Why `JSONEachRow`:
- it is simple
- it integrates naturally with `serde::Serialize`
- it keeps the row trait small
- it is a strong minimal choice for landing-table ingestion

We intentionally did not choose a more complex binary protocol or native format for the current v1 path because that would increase complexity without improving the architecture.

## Why Schema Ownership Lives In Decoder Crates

Schema ownership for concrete tables is decoder-owned, not writer-owned.

Implemented in the Jupiter path:
- `JupiterSwapClickHouseInstructionsMigration`
- `JupiterSwapSwapEventLandingRow::create_table_sql(...)`

Reasons:
- the decoder crate knows the semantic shape of the family best
- it mirrors Carbon's existing Postgres design
- it keeps generic ClickHouse core code free of decoder-specific schema logic
- it makes code generation possible
- it avoids hiding schema creation behind implicit first-write behavior

The generic core layer provides only the mechanism:
- `ClickHouseSchema`
- `ClickHouseAdmin`

The decoder decides what actual tables and DDL statements exist.

## Why We Introduced `ClickHouseRowContext`

Rows need some sink metadata that does not come from decoded blockchain data:
- `source_name`
- `mode`
- `decoder_version`

Originally, row generation depended on the full `ClickHouseConfig`. That was broader than necessary.

Now row generation depends on a smaller immutable context:
- `ClickHouseRowContext`

Reasons:
- row mapping should depend only on the metadata it actually needs
- it reduces coupling between row generation and transport settings
- decoder row modules remain focused on row semantics, not connection details
- it keeps the boundary between writer concerns and row concerns cleaner

## Current Row and Table Contracts

Core traits:
- `ClickHouseTable`
- `ClickHouseRow`
- `ClickHouseRows<R>`

Their roles are:

### `ClickHouseTable`
Defines table metadata and DDL for a row family:
- default table name
- columns
- create-table SQL

### `ClickHouseRow`
Defines runtime row behavior:
- serialization to one `JSONEachRow` line
- partition key selection

### `ClickHouseRows<R>`
Defines how a decoder-owned wrapper emits zero or more backend rows.

This is intentionally different from Postgres' `Insert`/`Upsert` trait model.

Why:
- a decoded instruction may produce zero rows, one row, or multiple rows
- ClickHouse is append-oriented here, not row-operation oriented
- row emission maps more naturally to event-family ingestion than one-row-at-a-time operations

## Why The Current Sink Is Landing-Only

The current implementation writes only to landing tables.

Reasons:
- landing tables are the smallest correct first step
- they let us validate the full path from datasource to ClickHouse without prematurely building the serving layer
- Carbon ingestion may include repeats, replays, and bounded backfills, so append-only landing storage is the safest initial contract
- serving/canonicalization introduces a different class of decisions about deduplication and replay resolution that are intentionally deferred

So the current sink proves:
- Carbon can decode real data
- Carbon can write that data to ClickHouse efficiently enough for a real pipeline
- landing rows contain the metadata needed for future serving logic

It does not yet try to solve the entire analytics warehouse problem.

## Current Jupiter Landing Table Design

Implemented row:
- `JupiterSwapSwapEventLandingRow`

Table:
- `jupiter_swap_swap_event_landing`

Stored fields include:
- identity fields
- chain metadata
- sink metadata
- typed payload columns for the swap event

Important columns:
- `program_id`
- `family_name`
- `event_type`
- `event_id`
- `slot`
- `signature`
- `instruction_index`
- `stack_height`
- `absolute_path`
- `event_seq`
- `source_name`
- `mode`
- `decoder_version`
- `ingest_ts`
- `chain_time`
- `partition_time`
- `block_hash`
- `tx_index`
- typed Jupiter swap payload fields

Engine choice:
- `MergeTree`

Partitioning:
- `PARTITION BY toYear(partition_time)`

Ordering:
- `ORDER BY (program_id, family_name, event_id, slot)`

Why this shape:
- keeps landing writes append-only
- stores enough metadata for later canonicalization work
- partitions by a durable time key without requiring chain time to always exist
- keeps the table family-specific and typed rather than forcing a generic universal event blob

## Identity and Replay Model

The current row identity uses deterministic event IDs.

Implemented helper:
- `deterministic_event_id(...)`

Inputs used by the Jupiter row:
- program id
- transaction signature
- instruction absolute path
- event type
- event sequence

For the current generated Jupiter CPI event path:
- `event_seq = 0`

Why:
- the current generated CPI event mapping emits one row for this event family per decoded instruction event
- the event identifier must remain stable across reprocessing
- a deterministic identity field is required if later serving or dedup logic is introduced

This is a landing-table contract, not yet a serving-table dedup policy.

## Metrics Design

The ClickHouse sink registers its own processor metrics with Carbon's upstream-v1 metrics registry.

Metrics tracked today:
- successful inserted rows
- failed rows in failed batches
- current buffered row count
- successful flush batch count
- failed flush batch count
- flush duration histogram

Why these metrics exist:
- ClickHouse inserts are batch-oriented, so row metrics alone are not enough
- buffer depth matters for an in-memory batched sink
- flush duration matters for diagnosing insert pressure and write latency
- failed batch metrics matter because failure scope is a whole batch, not one row

This is intentionally different from a row-at-a-time sink.

## Why The Example Uses Block Crawler

The example uses:
- `RpcBlockCrawler`

Reasons:
- it provides a deterministic bounded test path
- it is suitable for replaying a known slot range
- it is better for proving the landing-table ingest path than a purely live websocket feed
- it matches the kind of historical and backfill-oriented workload where ClickHouse is especially relevant

The example is not meant to demonstrate every datasource mode. It is meant to prove a realistic, controllable end-to-end integration.

## Why We Added Generator Support Early

The Jupiter ClickHouse integration could have remained handwritten.
It did not.

Initial ClickHouse renderer support was added because Carbon's long-term sink integrations are generator-backed. If ClickHouse remained purely handwritten, scaling it across decoders would diverge from the rest of the project.

Reasons for early generator support:
- it keeps ClickHouse aligned with Carbon's decoder generation model
- it reduces the risk that ClickHouse becomes a one-off path
- it makes the decoder-owned design reproducible for other programs
- it preserves architectural consistency with Postgres and GraphQL integrations

The current generation support is intentionally minimal. That is a scope choice, not an architectural rejection of broader generation.

## Known Tradeoffs and Non-Goals

### Tradeoffs we accepted
- adding processor finalization required touching core lifecycle code
- landing-only scope means the warehouse story is intentionally incomplete for now
- the sink currently supports a narrow decoder surface rather than the whole repository
- HTTP + `JSONEachRow` is simpler than more optimized formats, but not the ultimate performance ceiling

### Non-goals of the current implementation
- full ClickHouse parity with the Postgres sink surface
- account-family ClickHouse support
- serving-table canonicalization
- online dedup logic in the sink itself
- generic universal event tables
- replacing client batching with server-side async inserts

## Why This Design Was Chosen

The implemented design is the result of three practical constraints:

1. Carbon already has a strong processor-driven architecture.
2. ClickHouse needs a batched write model that is different from Postgres.
3. The first useful milestone is a real end-to-end landing sink, not a complete warehouse.

Given those constraints, this design was chosen because it provides:
- a real working ClickHouse integration in the current Carbon architecture
- a decoder-owned schema/mapping model that matches the rest of the project
- a ClickHouse-appropriate buffered runtime with safe shutdown behavior
- a code-generation path that can scale later
- a narrow but correct v1 slice that proves the architecture with real data

## Future Direction

The most likely next steps are:
- extend ClickHouse generation to broader instruction families
- add account-family ClickHouse support
- introduce additional decoder families beyond Jupiter swap
- add serving/canonicalization layers on top of landing tables
- evaluate later optimizations such as dedup tokens, query identifiers, or async insert tuning where justified

The current implementation should be viewed as the minimal correct ClickHouse foundation, not the final state.
