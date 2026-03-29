# ClickHouse Sink Implementation Plan

## Summary

Implement a ClickHouse sink that follows the same Carbon integration shape as the existing Postgres sink.

The sink remains intentionally narrow for V1:
- instruction and decoded event families only
- first concrete family: `jupiter_swap` `swap_event`
- landing-table writes only
- no serving tables, promotion jobs, or account-state handling yet

The implementation must still match the Carbon sink model in the important places:
- decoder-owned schema bootstrap
- processor lifecycle with guaranteed final drain
- row-semantic metrics compatible with Postgres operational expectations
- trait-driven wrapper dispatch from decoder crates into generic core processors

## Goals

V1 must prove all of the following:
- a real Carbon pipeline can decode Jupiter swap events and persist them to ClickHouse
- schema ownership lives in the decoder crate, not in the generic writer
- buffered rows are flushed before pipeline shutdown
- sink metrics distinguish row-level success/failure from batch-level flush behavior
- the example remains thin and close in shape to the Postgres example

## Architecture Decisions

### 1. Core lifecycle support

Add processor finalization to core.

Required changes:
- extend `Processor` with an async `finalize(&mut self, metrics)` hook
- default implementation is a no-op so existing processors do not change behavior
- extend pipe traits to expose `finalize(metrics)` and delegate to their processors
- make `Pipeline` call finalization before metric shutdown on all exit paths

Rationale:
- Postgres writes per item and does not buffer in memory
- ClickHouse V1 batches in memory, so shutdown without a final drain can drop accepted rows

### 2. Decoder-owned schema bootstrap

Move ClickHouse schema creation out of the generic writer.

Required changes:
- remove lazy `CREATE TABLE IF NOT EXISTS` from `ClickHouseBatchWriter`
- add a core ClickHouse admin/bootstrap utility for executing DDL
- each decoder sink module must expose explicit schema bootstrap helpers
- examples should call decoder-owned bootstrap before starting the pipeline, just as the Postgres example runs migrations before processing

Rationale:
- schema ownership should stay with the decoder crate
- schema rollout should not depend on first traffic hitting a table
- this keeps ClickHouse aligned with the Postgres migration pattern

### 3. Trait-driven instruction sink dispatch

Move ClickHouse instruction writes to the same generated wrapper style as Postgres.

Required changes:
- add a generic ClickHouse row-emission trait in core
- generic ClickHouse instruction processor should operate on wrapper types built from `(instruction, metadata, accounts)`
- decoder crates should provide wrapper types analogous to Postgres `InstructionWithMetadata`
- wrappers may emit zero, one, or many landing rows

V1 expectation:
- Jupiter wrapper emits rows only for `CpiEvent::SwapEvent`
- all other instruction variants return no rows

Rationale:
- Postgres integration is wrapper-driven and generated across decoder crates
- ClickHouse should follow the same pattern even if V1 covers only one family

### 4. Metrics semantics

Make ClickHouse metrics row-semantic first, batch-semantic second.

Required row metrics:
- `clickhouse.instructions.inserted`
- `clickhouse.instructions.failed`
- `clickhouse.instructions.buffered_rows` as a gauge

Required batch metrics:
- `clickhouse.instructions.flush.batches`
- `clickhouse.instructions.flush.failed_batches`
- `clickhouse.instructions.flush.duration_milliseconds`

Rules:
- successful flushes increment inserted rows by the number of rows written
- failed flushes increment failed rows by the number of rows in the attempted batch
- batch counters must not replace row counters

Rationale:
- Postgres metrics are row-semantic
- dashboards should remain comparable across sink backends
- ClickHouse-specific batch behavior still needs visibility

### 5. Writer scope

Keep the writer generic and narrow.

Writer responsibilities:
- buffer rows by partition
- flush on row threshold or flush interval
- preserve exact batch contents and ordering on retry
- expose explicit `flush()`
- report flushed row counts to the processor

Writer must not own:
- schema bootstrap
- decoder dispatch logic
- serving logic

### 6. Example shape

The example should remain as thin as the Postgres example.

The ClickHouse example should do only this:
- read env and CLI args
- bootstrap decoder-owned ClickHouse schema
- build the block crawler datasource
- build the decoder-owned ClickHouse processor
- wire the pipeline and run it

The example should not:
- construct DDL strings
- build row mapping closures
- parse ClickHouse connection details outside core helpers

### 7. Generator compliance

ClickHouse support must follow the renderer/codegen path used by Postgres instead of growing as handwritten decoder-specific modules only.

Required direction:
- keep client-side batching and core finalization support
- add ClickHouse-native core traits:
  - `ClickHouseTable`
  - `ClickHouseRows<R>`
  - `ClickHouseSchema`
- add renderer templates for generated ClickHouse instruction modules
- add render-map hooks so generated decoders can emit `src/instructions/clickhouse/...` alongside Postgres and GraphQL

V1 codegen scope:
- generate `src/instructions/clickhouse/mod.rs`
- generate `src/instructions/clickhouse/cpi_event_row.rs` when anchor events exist
- keep broader multi-family instruction/account parity as a follow-up once the core sink can target multiple generated row families cleanly

Rationale:
- this makes the ClickHouse path structurally compliant with Carbon's decoder generation model
- it avoids scaling a handwritten sink pattern across decoders
- it preserves the minimal V1 runtime while creating the correct long-term extension point

## Concrete V1 Targets

### Core

Under `crates/core/src/clickhouse/` maintain:
- `config.rs`
- `admin.rs` or equivalent schema/bootstrap helper
- `writer.rs`
- `rows/`
- `processors.rs`
- `mod.rs`

Public surface for V1:
- `ClickHouseConfig`
- `ClickHouseAdmin` or equivalent bootstrap helper
- `ClickHouseBatchWriter<R>`
- `ClickHouseInstructionProcessor<T, W, R>`
- generic ClickHouse row-emission trait

### Jupiter decoder

Under `decoders/jupiter-swap-decoder/src/instructions/clickhouse/` maintain:
- row modules for supported landing schemas
- wrapper type analogous to Postgres `InstructionWithMetadata`
- schema bootstrap type/helper analogous to Postgres migration ownership
- thin decoder-owned constructor helpers for processor setup

V1 support remains limited to:
- `JupiterSwapInstruction::CpiEvent(Box<CpiEvent::SwapEvent(_)))`

### Example

Keep one example:
- `examples/jupiter-swap-clickhouse`

Input path:
- `RpcBlockCrawler`
- `JupiterSwapDecoder`
- decoder-owned ClickHouse processor

Output path:
- `jupiter_swap_swap_event_landing`

## Landing Table Contract

Required columns for `jupiter_swap_swap_event_landing`:
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
- typed swap payload columns

Rules:
- `family_name = "jupiter_swap_swap_event"`
- `event_type = "swap_event"`
- `event_seq = 0` for the generated Jupiter CPI event decoder
- `event_id = hash(program_id, signature, absolute_path, event_type, event_seq)`
- `partition_time = chain_time if present, else ingest_ts`

DDL target:
- engine: `MergeTree`
- partition: `toYear(partition_time)`
- order by: `(program_id, family_name, event_id, slot)`

## Tests

### Unit tests

Required:
- stable `event_id`
- `event_seq == 0`
- stable Jupiter swap row mapping
- `partition_time` fallback behavior
- retry preserves exact batch contents and order
- final flush drains buffered rows

### Integration tests

Required:
- bootstrap schema explicitly before writes
- insert small batch into landing table succeeds
- row-threshold flush succeeds
- timer flush succeeds
- finalization flushes remaining rows
- repeated retry sends identical ordered rows

### Example validation

Required:
- run `jupiter-swap-clickhouse` with block crawler against a recent slot window
- verify landing table exists
- verify row count is non-zero
- verify sample rows contain expected swap payload fields

## Deferred work

Still deferred after this plan:
- account-side ClickHouse sink support
- serving tables and canonicalization jobs
- coverage tracking tables
- materialized views
- async inserts
- multi-family generated ClickHouse coverage across the entire Jupiter instruction surface

## Acceptance Criteria

This phase is complete when:
- ClickHouse sink uses decoder-owned schema bootstrap, not lazy writer DDL
- pipeline shutdown always finalizes ClickHouse processors before exit
- ClickHouse metrics expose row-level success/failure and batch-level flush behavior separately
- decoder-owned wrapper types drive ClickHouse row emission
- `examples/jupiter-swap-clickhouse` is as thin in responsibility as the Postgres example
- the example writes real Jupiter swap events into ClickHouse successfully
