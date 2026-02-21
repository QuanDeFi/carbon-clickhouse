# Carbon ClickHouse Sink Requirements (Draft)

## Goal

Build a production-safe ClickHouse sink for Carbon that is fast, replay-safe, and easy to operate.

V1 scope:
- Direct `Carbon -> ClickHouse` writes.
- ClickHouse is for analytics storage.

## Core Rule

Do **not** treat ClickHouse like Postgres.

Use:
- append-only writes,
- versioned rows for updates,
- tombstone rows for deletes.

Avoid in hot path:
- row-by-row `UPDATE`/`DELETE` mutations.

## Data Model

Use two table types:

1. Fact/Event tables
- For decoded immutable events (transfers, instructions, etc.).
- Engine: `MergeTree`.

2. State tables
- For “latest state” records (accounts, latest entity state).
- Engine: `ReplacingMergeTree(__ver, __is_deleted)`.
- Updates = insert same key with higher `__ver`.
- Deletes = insert same key with `__is_deleted = 1` and higher `__ver`.

Recommended key examples:
- Account key: `__pubkey`
- Instruction key: `__signature`, `__instruction_index`, `__stack_height`

## Ingestion Design

The sink must be batched and non-blocking.

Required:
- Internal buffer.
- Flush by thresholds:
  - max rows,
  - max bytes,
  - max time interval.
- Never insert one event at a time in steady state.

Why:
- Carbon pipeline is bounded; slow sink creates backpressure.
- ClickHouse performs best with batched inserts.

## Idempotency and Retries

Assume at-least-once delivery and retries.

Required:
- Stable event keys in rows.
- Retry-safe batch dedup token strategy.

Result:
- Replays and transient failures do not break correctness.

## Schema Evolution

Use a two-layer schema:
- Stage/raw ingest tables.
- Curated query tables (via materialized views).

Benefits:
- Safer schema changes.
- Cleaner query-facing types.
- Transform logic can move from sink code into ClickHouse SQL.

## Finality / Reorg Posture

V1 should:
- Prefer `confirmed` over `processed` for live feeds.
- Store enough lineage for corrections (`slot`, signature keys, optional block hash).
- Use eventual correction model (versioned inserts/tombstones), not destructive immediate rewrites.

## Carbon Integration Requirements

Add `clickhouse` feature in `crates/core`.

Create ClickHouse processors mirroring existing Postgres processor shape:
- `ClickHouseAccountProcessor<T, W>`
- `ClickHouseJsonAccountProcessor<T>`
- `ClickHouseInstructionProcessor<T, W>`
- `ClickHouseJsonInstructionProcessor<T>`

Error handling:
- Return `CarbonResult`.
- Map client/storage failures to `Error::Custom` consistently.

Metrics:
- `clickhouse.accounts.insert.inserted|failed|duration_milliseconds`
- `clickhouse.instructions.insert.inserted|failed|duration_milliseconds`
- Optional batch metrics (`rows`, `bytes`, `transactions`).

## Client Rules (`clickhouse-rs`)

Required:
- Always finish insert lifecycle (`end` / `commit`).
- Use configurable batching/timeouts.
- Keep validation on by default.
- If schema changes at runtime, clear cached insert metadata.

## Non-Goals for V1

- Heavy mutation-based correction workflows in hot path.

## Testing (Minimum)

1. Unit tests
- Row mapping and key/version/tombstone encoding.

2. Integration tests (with ClickHouse)
- Insert success for account + instruction flows.
- Replay/idempotency behavior.
- Tombstone behavior.
- Graceful shutdown flush (no lost buffered rows).

3. Basic performance checks
- Throughput and flush latency under live-like load.

## Implementation Order

1. Feature flags + module scaffolding.
2. Writer/batcher + config + dedup token policy.
3. Generic rows + DDL bootstrap.
4. Account/instruction processors + metrics.
5. Stage/raw + materialized-view curated path.
6. Tests + performance validation.
