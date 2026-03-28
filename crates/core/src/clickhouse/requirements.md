# Carbon ClickHouse Sink Requirements

## Goal

Build a direct Carbon -> ClickHouse sink for decoded Solana events that works for:
- live ingestion,
- historical backfills,
- snapshots,
- repair and replay jobs,
- arbitrary out-of-order inserts.

ClickHouse is the primary analytics store. This sink is event-first, not OLTP-style state storage.

## Decisions Already Locked

These are already supported by the Carbon codebase and should be treated as fixed.

- Input is decoded, not raw. Carbon processors already receive typed decoded outputs.
- Tables should be per event family, not one universal shared event table.
- The storage layout should be `landing -> serving`.
- The sink must support multiple ingestion modes:
  - `live`
  - `backfill`
  - `snapshot`
  - `repair`
- Coverage tracking is required because Carbon does not centrally track completeness across mixed datasource types.

## What Carbon Actually Provides

From the core metadata and datasource code:

- instruction-derived events have:
  - `signature`
  - `slot`
  - optional `block_time`
  - optional `block_hash`
  - instruction `index`
  - `stack_height`
  - `absolute_path`
- account updates have:
  - `pubkey`
  - `slot`
  - optional `transaction_signature`
- datasources already support:
  - live streaming,
  - bounded historical crawls,
  - snapshots,
  - backfill-then-tail patterns.

This means the sink must be replay-safe and must not assume ordered inserts.

## Decision Matrix

### Event Families

Decision:
- use one landing table and one serving table per decoded event family

Reason:
- decoder crates are already structured by program and event family
- schemas differ materially across programs
- per-family tables are easier to evolve and query

Examples:
- `jupiter_swap_swap_event_landing`
- `jupiter_swap_swap_event_serving`
- `solayer_restaking_deposit_event_landing`

### Event Identity

Decision:
- every row must have a deterministic `event_id`

Recommended identity inputs:
- `program_id`
- `signature`
- `absolute_path`
- `event_type`
- `event_seq`

Reason:
- Carbon provides `absolute_path`, which is stronger than only `instruction_index + stack_height` for CPI nesting
- Carbon does not provide a universal built-in event sequence for multiple decoded events emitted by one instruction

Required rule:
- `event_seq` must be assigned as the ordinal position within the decoded event list for one instruction

Recommended formula:
- `event_id = hash(program_id, signature, absolute_path, event_type, event_seq)`

### Landing Table Shape

Decision:
- landing tables should store decoded typed rows only

Required columns:
- `program_id`
- `event_type`
- `event_id`
- `slot`
- `block_time`
- `block_hash`
- `signature`
- `instruction_index`
- `stack_height`
- `absolute_path`
- `event_seq`
- `source_name`
- `mode`
- `decoder_version`
- `ingest_ts`
- typed payload columns

Optional:
- `payload_json` only if debug/replay ergonomics justify it

### Serving Table Shape

Decision:
- serving tables are canonical deduplicated event tables for reads

They should:
- preserve typed payload columns
- resolve duplicates from replays and overlapping backfills
- be query-facing

Optional later:
- add aggregate or rollup tables on top of serving tables

### Engine And Ordering

Decision:
- use `ReplacingMergeTree(...)` for both landing and serving if serving dedup is ClickHouse-native

Default starting point:
- `ReplacingMergeTree(version_column)`

`ORDER BY` must include the event identity key. Start with:
- `(program_id, event_type, event_id, slot)`

Reason:
- dedup in `ReplacingMergeTree` is based on the `ORDER BY` key

### Partitioning

Decision:
- default to yearly partitions, not monthly

Default:
- `PARTITION BY toYYYY(event_time)`

Important constraint from Carbon:
- `block_time` is not guaranteed on all update types

So the sink must define `event_time` explicitly:
- use `block_time` when present
- otherwise use a sink-defined fallback

This fallback still needs explicit sign-off.

### Insert Strategy

Decision:
- buffer rows in memory by partition key

Required behavior:
- partition-grouped buffers
- flush by row count
- flush by byte size
- flush by timer
- retry with exact same row order and exact same batch contents

Practical shape:
- `HashMap<partition_key, Vec<Row>>`

Backfill mode must use larger flush sizes than live mode.

### Coverage Tracking

Decision:
- add `ingestion_ranges`

Recommended columns:
- `program_id`
- `event_family`
- `source_name`
- `range_start_slot`
- `range_end_slot`
- `mode`
- `status`
- `started_at`
- `finished_at`
- `decoder_version`

Reason:
- Carbon supports mixed live, backfill, snapshot, and replay-style ingestion but does not track completeness centrally

## Open Decisions That Still Need Sign-Off

These could not be fully derived from Carbon code alone.

### 1. Version Rule

You need to choose what "latest row wins" means for serving-table dedup:

- highest `slot` wins
- latest `ingest_ts` wins
- latest `decoder_version` wins
- composite rule

This is the main unresolved semantic choice.

### 2. Event Time Fallback

You need to choose what `event_time` becomes when `block_time` is missing:

- derive externally from slot
- use `ingest_ts`
- keep `event_time` nullable and partition by another field

### 3. Serving Materialization Method

You need to choose how landing populates serving:

- ClickHouse materialized views
- scheduled promotion job
- both

My recommendation:
- start with a scheduled promotion job
- introduce materialized views only for simple stable transforms

## Carbon Integration Requirements

Add a `clickhouse` feature in `crates/core`.

Create ClickHouse processors that mirror Carbon's existing processor style:
- `ClickHouseAccountProcessor<T, W>`
- `ClickHouseJsonAccountProcessor<T>`
- `ClickHouseInstructionProcessor<T, W>`
- `ClickHouseJsonInstructionProcessor<T>`

The sink implementation must:
- return `CarbonResult`
- map storage failures to `Error::Custom`
- emit metrics for insert success, insert failure, flush duration, and batch size

## Client Rules

Using `clickhouse-rs`, the sink must:
- always complete insert lifecycle with `end` or `commit`
- keep validation enabled by default
- clear cached insert metadata after runtime schema changes
- configure batching and timeouts explicitly

## Minimum Tests

1. Unit tests
- `event_id` generation is stable
- row mapping is stable
- retry batches preserve exact row order
- `event_seq` assignment is stable for multi-event instructions

2. Integration tests
- live insert path works
- backfill insert path works
- snapshot ingest path works
- replayed rows converge correctly
- graceful shutdown flushes buffered rows
- coverage metadata is recorded correctly

3. Performance checks
- sustained live-like ingestion
- fragmented historical backfill ingestion
- flush latency and part creation remain acceptable

## Implementation Order

1. Confirm the three open decisions above.
2. Add row models and deterministic `event_id` / `event_seq` rules.
3. Add landing and serving DDL.
4. Build the partition-aware writer and retry policy.
5. Add coverage metadata tracking.
6. Add Carbon processors and metrics.
7. Test live, backfill, snapshot, and replay behavior.
