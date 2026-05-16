# ClickHouse Sink Architecture

## Purpose

This document explains the architecture and design boundaries of the Carbon ClickHouse sink. It focuses on why the sink is shaped this way, what invariants should remain stable, and which responsibilities intentionally stay outside the sink.

For exact modules, configuration fields, generated file paths, validation commands, monitoring queries, and example operation, see `Carbon Sink Implementation.md`.

## Architecture Summary

The ClickHouse sink is a generator-backed typed landing-table backend for Carbon. It follows Carbon's existing processor model and keeps concrete schema ownership in decoder crates.

The current architectural scope is:

- decoder-owned account, instruction, and CPI/event landing rows
- one typed landing table per generated row family
- append-only landing writes
- deterministic landing row identity
- client-side batched ClickHouse inserts
- explicit shutdown drain through processor finalization
- synchronous inserts by default
- optional async-wait inserts for live multi-writer deployments
- renderer-controlled production DDL modes

The sink is not a warehouse serving layer. Serving tables, canonicalization, durable replay control, slot/range coverage, finality policy, DLQs, and APIs belong above or around the sink.

## Core Design Principles

The sink is built around these principles:

- Fit Carbon's processor-driven architecture instead of introducing a separate sink subsystem.
- Keep decoder-owned schema and row mapping in decoder crates.
- Treat IDL/Codama/decoder schema as the source of truth for known program data.
- Use ClickHouse in a ClickHouse-appropriate way: batched, append-oriented, and partition-aware.
- Preserve replay-safe deterministic row identity without pretending ClickHouse primary keys enforce uniqueness.
- Keep the first-class sink contract landing-only and avoid hiding serving/canonicalization semantics inside ingestion.
- Keep generated ClickHouse output feature-gated and rollout-controlled.

## Solana Schema Boundary

Carbon's decoder-owned data families map to Solana program artifacts:

- account layouts
- instruction layouts
- CPI/event layouts
- shared IDL/Codama-defined types

That is why ClickHouse table schemas are decoder-owned. The sink should not infer table structure from live RPC rows, Geyser rows, account bytes, transaction payloads, or logs.

Live chain data is untrusted input until the decoder validates it. The validation boundary includes owner checks, data length checks, account and instruction discriminators, instruction account arrangement, and event discriminator conventions where applicable.

Known decoder-owned schema should become typed ClickHouse columns. Unsupported known schema should fail generation unless an explicit renderer fallback is enabled. Silent JSON fallback is a schema-quality bug for known program shapes.

## CPI/Event Model

CPI/events are treated as decoder-owned instruction-family output, not as a separate global Carbon persistence family.

Reasoning:

- Reliable indexer events can be emitted as CPI instruction data on the program itself.
- Program logs can truncate and should not be the only source of indexed event truth.
- Carbon already decodes these events through instruction/event decoder logic.
- The event schema is still owned by the decoder, not by the generic sink runtime.

This keeps ClickHouse aligned with the same decoder-owned families used by the Postgres integration: accounts, instructions, and events through the instruction path.

## Where ClickHouse Fits In Carbon

The ClickHouse integration is a normal Carbon processor path.

Conceptually:

1. A datasource emits Carbon updates.
2. Carbon routes updates into account or instruction pipes.
3. Decoders validate and decode program-owned data.
4. A ClickHouse processor receives normal decoded processor input.
5. Decoder-owned ClickHouse code maps decoded data into typed row families.
6. A generic ClickHouse writer batches and inserts rows.

The pipeline does not need a special ClickHouse mode. Processors own side effects, and decoder crates own backend-specific row mapping.

## Architectural Layers

The sink has four layers:

- `carbon-core` ClickHouse runtime: connection configuration, HTTP transport, schema execution, row/table contracts, buffering, retries, metrics, and processor integration.
- Decoder-owned ClickHouse modules: program-specific row mapping, table definitions, migrations, and setup helpers.
- Renderer support: generated typed row modules and DDL planning for account, instruction, and CPI/event families.
- Examples: thin end-to-end wiring that proves real datasource-to-ClickHouse ingestion without embedding schema logic in examples.

The important boundary is that the core runtime knows how to write rows to ClickHouse, but it does not know Jupiter, Token Program, or any other program-specific schema.

## Relationship To Postgres

ClickHouse follows the same high-level Carbon integration model as Postgres:

- processor-driven execution
- decoder-owned schema and row mapping
- generated backend modules
- thin examples

The write model intentionally differs.

Postgres is row-operation oriented. ClickHouse is append-oriented and performs best with fewer, larger inserts. A ClickHouse sink that writes one row at a time like Postgres would create unnecessary insert pressure and poor MergeTree part behavior.

Therefore the ClickHouse sink adds buffering, per-buffer flushing, retry/backoff, byte-aware backpressure, and shutdown drain. Those are not deviations from Carbon's model; they are the ClickHouse-specific runtime mechanics needed to preserve that model safely.

## Typed Landing Table Model

The standard table model is one typed landing table per generated row family.

Reasons:

- Decoder-owned schemas remain explicit and strongly typed.
- ClickHouse compression and query planning work better with stable typed columns than one universal JSON blob.
- Family-specific tables keep instruction, event, and account payloads queryable without downstream JSON parsing.
- Landing rows can still be appended safely during replay or backfill.
- Serving/canonicalization logic can be built later without changing the ingestion contract.

The model intentionally avoids:

- one global generic landing table for all decoder data
- implicit schema inference from first row
- row-level online deduplication in the ingest path
- serving tables as part of the Carbon sink

## Identity And Replay Model

Rows use deterministic landing IDs derived from stable chain and decoder metadata.

The ID contract exists because Carbon workloads include replays, bounded backfills, repeated processing, and many independent Carbon processes writing into the same ClickHouse deployment.

Deterministic IDs provide stable identity metadata for later deduplication, canonicalization, or serving-table construction. They do not make ClickHouse inserts unique by themselves, because ClickHouse primary keys do not enforce uniqueness.

The sink contract is:

- write append-only landing rows
- include deterministic identity columns
- preserve enough metadata for downstream canonicalization
- leave replay convergence policy outside the sink

## Batching And Shutdown

ClickHouse ingestion is buffered in the Carbon process.

Architectural reasons:

- ClickHouse prefers batched inserts.
- Carbon processors provide a natural boundary for grouping decoded rows.
- Backfills benefit from deterministic batch construction and explicit insert outcomes.
- Local buffering allows safe shutdown drain before the pipeline exits.
- Per-buffer flushing prevents one hot family from forcing unrelated cold tables to flush early.

Because rows can remain in memory after processing, processor finalization is required. The pipeline must give processors a shutdown hook so buffered sinks can flush or surface errors before the process exits. Relying on timers or `Drop` for async I/O would be incorrect.

## Insert Semantics

Synchronous inserts are the default because they are easiest to reason about for backfills, bounded crawls, and deterministic replay jobs.

Async inserts are supported only as async-wait inserts. The sink intentionally does not expose fire-and-forget `wait_for_async_insert=0`, because that would hide server-side insert failures from the Carbon process.

This gives two intended modes:

- sync inserts for deterministic ingestion and backfills
- async-wait inserts for production live ingestion with many Carbon writers, where ClickHouse can coalesce writes server-side while Carbon still observes acknowledgement

## DDL Ownership

Concrete landing-table DDL is decoder-owned and renderer-generated.

The core runtime provides schema execution primitives. It does not decide which tables a program owns or what columns they contain.

Renderer-controlled DDL modes exist so the same generated row families can be deployed as simple local `MergeTree` tables, replicated tables, or distributed table setups. The renderer may generate additive schema operations, but destructive type-change migrations remain outside the sink.

## Metrics Model

ClickHouse sink metrics focus on processor-family and writer health rather than per-row database operations.

The useful signals are:

- inserted and failed rows
- inserted and failed bytes
- buffered rows and bytes
- active buffers
- flush batch counts
- flush latency
- retries
- backpressure rejections

Metrics are aggregated by ClickHouse processor family in the current Carbon metrics infrastructure. This matches the label-less upstream-v1 registry and keeps the sink observable without introducing a new metrics model.

Production process-level visibility should combine:

- Carbon pipeline metrics
- ClickHouse sink metrics
- ClickHouse server-side system tables and query logs
- deployment-level process/container labels from the monitoring stack

## Responsibility Split

The Carbon sink handles:

- typed conversion from decoder-owned account, instruction, and CPI/event schemas
- deterministic landing IDs
- local row/byte batching and per-buffer flushing
- shutdown drain through processor finalization
- sync default inserts and async-wait insert settings
- transient HTTP retry/backoff and failed-buffer preservation
- optional exact-batch insert deduplication tokens
- generated landing-table bootstrap
- sink-side metrics

ClickHouse handles:

- physical storage
- compression
- partitioning, sorting, and merges
- replicated and distributed topology
- server-side async insert batching when configured
- materialized views and downstream serving tables
- retention and query-time performance
- eventual dedup or canonicalization engines above landing tables, if selected

External control-plane or application code handles:

- durable queues
- durable retry state
- slot/range coverage tracking
- replay orchestration
- source checkpoints
- DLQs and poison-record policy
- Solana finality and reorg policy
- schema rollout coordination
- serving API semantics

Those control-plane responsibilities intentionally do not live inside `carbon-core::clickhouse`.

## Rollout Boundary

Generated ClickHouse output is intentionally canary-limited in this repository.

The architecture supports broader decoder generation, but committed output remains limited until upstream v1 stabilizes and broader decoder validation is intentionally rolled out.

The current canary boundary validates:

- Jupiter swap instruction and CPI/event rows
- Token Program account rows

That boundary is a release-risk decision, not a limitation of the architecture.

## Non-Goals

The ClickHouse sink does not implement:

- serving or canonicalization tables
- online row-level deduplication
- durable retry queues
- DLQs
- slot/range coverage tracking
- replay orchestration
- Solana finality policy
- serving APIs
- destructive schema migration generation
- universal JSON landing tables
- fire-and-forget async inserts
- committed broad decoder regeneration before upstream v1 stabilizes

## Architectural Invariants

Future changes should preserve these invariants unless the architecture is intentionally revised:

- Decoder-owned schema remains the source of truth for generated ClickHouse tables.
- Known decoder-owned shapes should stay typed, not silently stringified or moved to JSON.
- ClickHouse core runtime remains decoder-agnostic.
- Examples remain thin and should not contain program-specific ClickHouse row mapping.
- The sink remains landing-first and append-oriented.
- Processor finalization remains required for buffered write correctness.
- Synchronous inserts remain the default.
- Async insert support remains acknowledgement-preserving.
- Durable replay, finality, DLQ, and serving policy remain outside the sink.
