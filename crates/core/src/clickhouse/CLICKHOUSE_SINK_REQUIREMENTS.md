# Carbon ClickHouse Sink Requirements

## Goal

Build a direct Carbon -> ClickHouse sink for decoded Solana events that works for:
- live ingestion,
- historical backfills,
- arbitrary out-of-order inserts,
- replay and repair jobs.

ClickHouse is the primary analytics store. This sink is event-first, not OLTP-style state storage.

## Core Rule

Do not treat ClickHouse like Postgres.

The sink must be:
- append-first,
- replay-safe,
- batch-oriented,
- tolerant of temporary duplicates.

Avoid in the hot path:
- row-by-row inserts,
- row-by-row `UPDATE` or `DELETE`,
- fine-grained partitions.

## Data Model

Store decoded events as canonical event rows.

Recommended table shape:
- one raw table per event family, such as `swaps_raw`, `fills_raw`, `liquidations_raw`
- or one shared `decoded_events_raw` table if shared infrastructure is more important than per-family specialization

Required columns:
- `program_id`
- `event_type`
- `event_id`
- `slot`
- `event_time`
- `signature`
- `instruction_index`
- `inner_index`
- `event_seq`
- `decoder_version`
- `ingest_ts`
- typed payload columns
- optional `payload_json`

## Event Identity

Every decoded event must have a deterministic `event_id`.

It must be derived only from stable chain/decoder fields, for example:
- `signature`
- `instruction_index`
- `inner_index`
- `event_type`
- `event_seq`
- optionally `program_id`

This is the main protection against replay duplication and backfill overlap.

## ClickHouse Table Design

Best default engine:
- `ReplacingMergeTree(ingest_ts)`

Best default partitioning:
- `PARTITION BY toYYYY(event_time)`

Best default ordering:
- include the event identity columns in `ORDER BY`
- for a shared table, start with `(program_id, event_type, event_id, slot)`

Why:
- yearly partitions are coarse enough for fragmented backfills
- `ReplacingMergeTree` gives eventual deduplication for replayed events
- the `ORDER BY` key is the dedup key for ReplacingMergeTree

Do not default to monthly partitions unless backfill behavior is tightly controlled.

## Insert Strategy

The sink must buffer rows in memory by partition key.

Required behavior:
- maintain partition-grouped buffers
- flush by row count
- flush by byte size
- flush by timer
- retry using the exact same row order and exact same batch contents

Practical shape:
- `HashMap<partition_key, Vec<Row>>`

Reason:
- ClickHouse performs poorly when a single workload creates many tiny parts across many partitions
- backfills will naturally scatter events across history unless the sink actively groups them

Use larger flush sizes for backfill than for live ingestion.

## Deduplication and Retries

There are two separate duplication cases.

1. Retry duplication
- same batch retried after failure
- protect this with stable batch contents, stable row order, and a batch identifier / insert token

2. Replay or backfill duplication
- old history is intentionally reprocessed
- protect this with deterministic `event_id` plus `ReplacingMergeTree`

Do not rely only on ClickHouse retry deduplication windows. They are not a permanent semantic dedup layer.

## Query Model

Do not assume raw tables should always be queried directly.

Preferred query patterns:
- canonical reads with `argMax(..., ingest_ts)` or equivalent
- `FINAL` only for narrow validation queries

Optional later optimization:
- introduce a second serving/canonical table if analyst queries become sensitive to duplicates or ingestion turbulence

## Schema Evolution

Use a two-layer layout when transforms or schema churn justify it:
- landing/raw table for ingest
- serving/curated table for queries

Population options:
- materialized views
- controlled post-ingest jobs

This is the right place to isolate ingest turbulence from query ergonomics.

## Operational Contract

The sink should expose four clear behaviors:
- `write_live(events)`
- `write_backfill(events, range_metadata)`
- `retry_batch(batch_id)`
- `record_coverage(range)`

Backfill mode should use:
- larger batches
- partition-aware grouping
- stable ordered retries

## Coverage Metadata

Add a small metadata table to track ingestion coverage.

Recommended table: `ingestion_ranges`

Columns:
- `program_id`
- `source_name`
- `range_start_slot`
- `range_end_slot`
- `mode`
- `status`
- `started_at`
- `finished_at`
- `decoder_version`

Modes should at least include:
- `live`
- `backfill`
- `repair`

This is required to know what history is complete and what can be replayed safely.

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

2. Integration tests
- live insert path works
- backfill insert path works
- replayed rows converge correctly
- graceful shutdown flushes buffered rows
- coverage metadata is recorded correctly

3. Performance checks
- sustained live-like ingestion
- fragmented historical backfill ingestion
- flush latency and part creation remain acceptable

## Implementation Order

1. Feature flags and module scaffolding.
2. Row model and deterministic `event_id` rules.
3. Partition-aware writer/batcher.
4. Retry policy and coverage metadata table.
5. Raw table DDL and optional landing/serving flow.
6. Carbon processors and metrics.
7. Tests and performance validation.
