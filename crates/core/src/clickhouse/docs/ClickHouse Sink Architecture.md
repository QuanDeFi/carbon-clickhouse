# ClickHouse Sink Architecture

## Purpose

This document explains the architecture and design boundaries of the Carbon ClickHouse sink. It focuses on why the sink is shaped this way, what invariants should remain stable, and which responsibilities intentionally stay outside the sink.

For exact modules, configuration fields, generated file paths, validation commands, monitoring queries, and example operation, see `Carbon Sink Implementation.md`.

## Architecture Summary

The ClickHouse sink is a generator-backed typed landing-table backend for Carbon. It follows Carbon's existing processor model and keeps concrete schema ownership in decoder crates.

The current architectural scope is:

- decoder-owned account, instruction, and CPI/event landing rows
- typed landing tables: one per instruction family, one per account family, and one per generated CPI/event family
- append-only landing writes
- deterministic landing row identity
- client-side batched ClickHouse inserts
- explicit shutdown drain through processor finalization
- synchronous inserts by default
- optional async-wait inserts for live multi-writer deployments
- renderer-controlled production DDL modes
- pre-ingest schema drift validation with safe enum-extension repair
- CLI parity for renderer DDL options through `--clickhouse-options`

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

The physical ClickHouse table shape is intentionally not identical to upstream
Postgres for CPI/events. Upstream Postgres groups all CPI/event variants into a
generic JSON-backed event table. The current ClickHouse sink ships one typed
landing table per generated CPI/event family, such as
`jupiter_swap_fee_event_landing`, `jupiter_swap_swap_event_landing`, and
`jupiter_swap_swaps_event_landing`. The temporary unified
`jupiter_swap_cpi_event_landing` table was removed and is stale local state if
it still exists.

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

## Carbon Core Touch Points

The ClickHouse sink is not a new Carbon pipeline abstraction. It does, however,
require a few shared Carbon/runtime touch points so buffered writes are safe in
real pipelines:

- `carbon-core::clickhouse` contains the generic runtime for HTTP transport,
  schema execution, buffering, retries, deduplication settings, metrics, and
  account/instruction processor integration.
- `Processor::finalize()` and pipeline finalization are used to drain buffered
  rows before shutdown completes.
- The RPC block crawler datasource is hardened for live ClickHouse examples
  because datasource loss happens before the sink can retry anything.

The RPC block crawler hardening is specifically about finalized block-stream
stability:

- retry temporary near-head `getBlock` failures such as `BlockNotAvailable`
  and `BlockStatusNotAvailableYet`;
- keep true permanent skipped/cleaned slots as skipped and log the extracted
  RPC error code;
- use awaited downstream sends so a full Carbon channel backpressures instead
  of dropping transaction updates;
- let queued fetched blocks drain before the crawler task exits.

This is why the branch touches a datasource crate even though ClickHouse row
mapping remains decoder-owned. The sink can only guarantee retry and shutdown
drain for rows that reach the ClickHouse processor.

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

The standard table model is typed landing tables: one table per instruction
family, one table per account family, and one table per generated CPI/event
family.

Reasons:

- Decoder-owned schemas remain explicit and strongly typed.
- ClickHouse compression and query planning work better with stable typed columns than one universal JSON blob.
- Instruction, account, and CPI/event family tables keep payloads queryable without downstream JSON parsing.
- Per-event tables preserve the structured ClickHouse mapping for each known
  event payload instead of collapsing all events back into a Postgres-style
  generic JSON table.
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

## Multi-Writer Model

The production scaling model is many independent Carbon processes writing to
the same ClickHouse deployment. Each process owns its own datasource,
processor, local buffers, retry loop, and shutdown drain. The sink does not
introduce a global writer registry or cross-process coordinator.

ClickHouse is made "aware" of Carbon writes through standard request settings
and row metadata:

- every insert attempt has a Carbon-shaped `query_id` for query-log and
  async-insert-log tracing;
- exact-batch insert deduplication tokens are emitted by default so retrying
  the same serialized batch can be idempotent when the table engine supports
  insert deduplication;
- async-wait insert settings let ClickHouse coalesce inserts from many writers
  server-side without hiding insert failures from Carbon;
- landing rows include `source_name`, `mode`, and `decoder_version`, so
  downstream queries can separate pipelines, sources, backfills, snapshots, and
  live ingestion.

Those controls are not a substitute for a control plane. Durable queueing,
source checkpoints, replay scheduling, per-process deployment labels, and
range coverage tracking remain outside the sink.

## DDL Ownership

Concrete landing-table DDL is decoder-owned and renderer-generated.

The core runtime provides schema execution primitives. It does not decide which tables a program owns or what columns they contain.

Renderer-controlled DDL modes exist so the same generated row families can be deployed as simple local `MergeTree` tables, replicated tables, or distributed table setups.

Generated managed schema metadata is the source of truth for landing-table shape. Before ingestion, the runtime compares generated expected column definitions with live ClickHouse tables. Missing tables and columns are additive and safe. Enum-extension-only drift is repaired automatically because the generated enum keeps all existing values and only adds new variants with stable numeric IDs.

Unsafe drift is not hidden. Scalar type changes, tuple shape changes, removed enum values, changed enum numeric IDs, engine drift, partition drift, order-key drift, and unclassified mismatches fail before rows are inserted.

Destructive table repair and production type-change migration remain outside the ingestion path. The sink detects the problem, but table recreation and broader migration planning require explicit operator-controlled admin work.

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

The current metrics registry is label-less. The sink therefore exposes
processor-family aggregates rather than per-table or per-pipeline labeled
metrics. In multi-process deployments, per-process and per-pipeline dashboards
should come from the scrape target, service name, container labels, and row
metadata rather than from dynamic metric labels inside Carbon.

## Responsibility Split

The Carbon sink handles:

- typed conversion from decoder-owned account, instruction, and CPI/event schemas
- deterministic landing IDs
- local row/byte batching and per-buffer flushing
- shutdown drain through processor finalization
- sync default inserts and async-wait insert settings
- transient HTTP retry/backoff and failed-buffer preservation
- exact-batch insert deduplication tokens, enabled by default for retry
  idempotency
- generated managed schema bootstrap
- safe generated-schema drift validation and enum-extension repair
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

The architecture supports broader decoder generation, but committed output remains limited while broader decoder validation and rollout remain deliberate future work.

The current canary boundary validates:

- Jupiter swap instruction and CPI/event rows
- Jupiter swap TokenLedger account rows in the live head-follow example path
- Token Program account and instruction rows

The committed canaries are renderer-generated and use generated managed schema
metadata for bootstrap and drift validation. That boundary is a release-risk
decision, not a limitation of the architecture.

## Roadmap

Near-term schema-evolution follow-up work:

- Add schema version or schema fingerprint tracking for generated landing-table
  metadata. The goal is to make expected/generated schema state observable
  across deployments, predeploy validation, and live ingestion diagnostics.
- Split decoder provenance metadata from the current logical `decoder_version`
  field. `decoder_version` should represent the renderer `versionName` or
  equivalent decoder-routing version, while separate metadata should track the
  decoder crate version, IDL/schema fingerprint, generator version, and optional
  generated-from commit for traceability.
- Add table-versioning or side-by-side table coexistence for breaking generated
  schema changes. This is the safer path when old and new decoder versions need
  to run at the same time or when a change is broader than additive columns or
  enum-extension-only drift.

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
- destructive drop/recreate repair during ingestion startup
- universal JSON landing tables
- fire-and-forget async inserts
- committed broad decoder regeneration without explicit rollout validation

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
