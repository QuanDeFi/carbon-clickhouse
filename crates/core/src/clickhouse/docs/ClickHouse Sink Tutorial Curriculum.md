# ClickHouse Sink Tutorial Curriculum

## Purpose

This curriculum teaches engineers how to use Carbon's ClickHouse sink to write
decoded Solana data into typed ClickHouse landing tables. It is organized as a
hands-on tutorial path, starting with the sink model and ending with local
validation, observability, and production boundaries.

The curriculum uses the current canary examples:

- `examples/jupiter-swap-clickhouse` for instruction and CPI/event landing rows.
- `examples/token-program-clickhouse` for account landing rows.

## Audience

The intended audience is engineers who:

- know basic Rust and Cargo workflows
- understand Solana accounts, instructions, and RPC at a working level
- want to index decoded Solana data into ClickHouse
- need to understand what the sink guarantees and what must be built around it

## Prerequisites

Before starting, learners should have:

- this repository checked out locally
- a working Rust toolchain
- Docker access for the local ClickHouse and monitoring services
- a provider RPC URL for Solana mainnet data
- optionally, a Helius RPC URL for the Token Program account snapshot example

Do not use the public mainnet-beta RPC endpoint for the examples. These smoke
tests need a production/provider RPC endpoint.

## Learning Outcomes

After completing the curriculum, learners should be able to:

- explain where ClickHouse fits in Carbon's datasource, decoder, processor, and
  writer flow
- run the Jupiter ClickHouse example in bounded and live modes
- run the Token Program ClickHouse account snapshot example with safe filters
- identify the generated landing tables created by each example
- validate inserted rows with ClickHouse SQL
- distinguish synchronous inserts from async-wait inserts
- inspect ClickHouse sink metrics through the local Prometheus stack
- explain the sink's boundaries around replay, deduplication, canonicalization,
  finality, durable queues, and serving APIs

## Module 1: Architecture Primer

Goal: understand the sink's role before running examples.

Topics:

- Carbon datasource to decoder to processor flow.
- ClickHouse as a normal Carbon processor path, not a separate pipeline mode.
- Decoder-owned schema and generated ClickHouse row mapping.
- Typed landing tables for accounts, instructions, and CPI/events.
- Append-only landing writes and deterministic row identifiers.
- Why ClickHouse inserts are buffered and flushed in batches.
- Why processor finalization is required for shutdown drain.

Hands-on reading:

- `crates/core/src/clickhouse/docs/ClickHouse Sink Architecture.md`
- `crates/core/src/clickhouse/docs/Carbon Sink Implementation.md`

Checkpoint questions:

- Which layer owns concrete table schemas?
- Why does the sink write landing tables instead of serving tables?
- Why do deterministic IDs not enforce uniqueness by themselves in ClickHouse?

## Module 2: Local Environment Setup

Goal: confirm that ClickHouse and monitoring are reachable before running
Carbon examples.

Commands:

```sh
docker ps --format '{{.Names}} {{.Image}} {{.Ports}}' | rg 'clickhouse|carbon-prometheus|carbon-grafana'
curl -fsS 'http://carbon:carbon@localhost:8123/?query=SELECT%201'
docker compose -f monitoring/compose.yaml up -d
docker compose -f monitoring/compose.yaml ps
```

Expected outcome:

- ClickHouse responds to `SELECT 1`.
- Prometheus is available at `http://localhost:9090`.
- Grafana is available at `http://localhost:3000`.
- The Carbon examples can expose metrics at `0.0.0.0:9464/metrics`.

Troubleshooting focus:

- ClickHouse container not running.
- Port conflicts for Prometheus, Grafana, or the Carbon metrics endpoint.
- Provider RPC URL missing from the example `.env`.

## Module 3: Jupiter Instruction And CPI/Event Ingestion

Goal: run real Solana blocks through the generated Jupiter Swap instruction and
CPI/event ClickHouse path.

Example directory:

```sh
cd examples/jupiter-swap-clickhouse
```

Environment:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=<provider-rpc-url>
BLOCK_CRAWLER_START_SLOT=<start-slot-or-empty>
BLOCK_CRAWLER_END_SLOT=<end-slot-or-empty>
BLOCK_CRAWLER_HEAD_LAG_SLOTS=3
PROMETHEUS_METRICS_ADDR=0.0.0.0:9464
LOG_LEVEL=info
```

Run:

```sh
cargo run -p jupiter-swap-clickhouse-carbon-example
```

Modes:

- Set both `BLOCK_CRAWLER_START_SLOT` and `BLOCK_CRAWLER_END_SLOT` for a
  bounded backfill.
- Set `BLOCK_CRAWLER_START_SLOT` and leave `BLOCK_CRAWLER_END_SLOT` empty to
  catch up from that slot and keep following head.
- Leave both slot envs empty for pure head-follow mode. The example starts near
  the current confirmed slot using `BLOCK_CRAWLER_HEAD_LAG_SLOTS`.

Teaching notes:

- The example bootstraps generated Jupiter instruction landing tables.
- It also bootstraps one generated Jupiter CPI/event landing table.
- In pure head-follow mode, it attaches the generated TokenLedger account
  processor and fetches current confirmed TokenLedger account snapshots for
  first-seen TokenLedger pubkeys.
- TokenLedger account fetching is live-only because normal RPC account reads
  return current confirmed account state, not historical state for an old
  backfill slot.
- Empty instruction tables are expected when the selected slot window does not
  contain that instruction type.

Validation queries:

```sql
SHOW TABLES FROM default LIKE 'jupiter_swap_%landing';

SELECT event_type, count()
FROM default.jupiter_swap_cpi_event_landing
GROUP BY event_type
ORDER BY event_type;

SELECT
  count() AS rows,
  uniq(signature) AS signatures
FROM default.jupiter_swap_route_instruction_landing;
```

Expected outcome:

- At least one Jupiter landing table exists.
- The unified CPI/event table exists as `jupiter_swap_cpi_event_landing`.
- If the selected blocks include decoded Jupiter activity, row counts increase.
- Sink buffers drain on shutdown.

## Module 4: Token Program Account Snapshot Ingestion

Goal: run a bounded account-family ingestion path into generated Token Program
ClickHouse landing tables.

Example directory:

```sh
cd examples/token-program-clickhouse
```

Environment:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=<provider-rpc-url>
HELIUS_RPC_URL=https://mainnet.helius-rpc.com/?api-key=<key>
TOKEN_ACCOUNT_OWNER=<wallet-pubkey>
# TOKEN_MINT=<mint-pubkey>
LOG_LEVEL=info
PROMETHEUS_METRICS_ADDR=0.0.0.0:9464
```

At least one account filter must be set:

- `TOKEN_ACCOUNT_OWNER`, matched at token account data offset 32.
- `TOKEN_MINT`, matched at token account data offset 0.

Run with the default Helius gPA v2 source:

```sh
cargo run -p token-program-clickhouse-carbon-example
```

Run with standard Solana JSON-RPC `getProgramAccounts`:

```sh
cargo run -p token-program-clickhouse-carbon-example -- --source rpc
```

Teaching notes:

- The default path focuses on token accounts because owner and mint filters keep
  the query bounded.
- The generated decoder also supports mint and multisig account landing rows.
- Unbounded Token Program account scans are intentionally not a safe default.
- GPA snapshots usually have `transaction_signature = NULL` because they are
  not tied to a single transaction.

Validation queries:

```sql
SHOW TABLES FROM default LIKE 'token_program_%account_landing';

SELECT
  count() AS rows,
  uniq(pubkey) AS accounts,
  min(slot) AS min_slot,
  max(slot) AS max_slot
FROM default.token_program_token_account_landing;
```

Expected outcome:

- The Token Program account landing tables exist.
- Filtered token account rows land in
  `token_program_token_account_landing`.
- Mint and multisig landing tables can exist even when they are empty for a
  token-account-only run.

## Module 5: Insert Modes And Runtime Configuration

Goal: understand how the sink sends batches to ClickHouse.

Topics:

- Synchronous inserts are the default for deterministic backfills and bounded
  crawls.
- Async-wait inserts are intended for live multi-writer deployments where
  ClickHouse should coalesce inserts server-side while Carbon still waits for
  acknowledgement.
- Fire-and-forget async inserts are intentionally not exposed.
- Batch settings control local row and byte buffering.
- Retry settings handle transient HTTP failures.
- Transport settings control request timeout, connection timeout, pooling,
  compression, and user agent.
- Optional exact-batch deduplication tokens can be enabled by callers that need
  exact insert-batch identity.

Jupiter async-wait run:

```sh
CLICKHOUSE_ASYNC_INSERT=true cargo run -p jupiter-swap-clickhouse-carbon-example
```

Token Program async-wait run:

```sh
CLICKHOUSE_ASYNC_INSERT=true \
CLICKHOUSE_ASYNC_INSERT_BUSY_TIMEOUT_MS=1000 \
cargo run -p token-program-clickhouse-carbon-example
```

ClickHouse async insert inspection:

```sql
SYSTEM FLUSH LOGS;

SELECT
  event_time,
  table,
  rows,
  bytes,
  status,
  query_id,
  flush_query_id
FROM system.asynchronous_insert_log
WHERE event_time >= now() - INTERVAL 15 MINUTE
ORDER BY event_time DESC;
```

Checkpoint questions:

- When should a backfill use sync inserts?
- Why does async-wait preserve acknowledgement semantics?
- What should happen to buffered rows during shutdown?

## Module 6: Observability And Health Checks

Goal: validate that the Carbon process and ClickHouse sink are healthy.

Start monitoring:

```sh
docker compose -f monitoring/compose.yaml up -d
```

Inspect the Carbon metrics endpoint:

```sh
curl -sS http://localhost:9464/metrics | rg 'carbon_updates_failed_total|clickhouse_'
```

Prometheus queries:

```promql
carbon_updates_queued
rate(carbon_updates_processed_total[1m])
rate(carbon_updates_failed_total[1m])
clickhouse_instructions_buffered_rows
rate(clickhouse_instructions_flush_failed_batches[1m])
clickhouse_accounts_buffered_rows
rate(clickhouse_accounts_flush_failed_batches[1m])
```

Healthy signals:

- `carbon_updates_failed_total` stays at zero for the smoke run.
- `clickhouse_instructions_flush_failed_batches` stays at zero for Jupiter.
- `clickhouse_accounts_flush_failed_batches` stays at zero for account runs.
- Buffered row gauges return to zero after finite shutdown.
- Retry rates do not grow continuously.

Troubleshooting focus:

- RPC provider timeouts or unsupported methods.
- Missing or malformed `.env` values.
- ClickHouse authentication or endpoint errors.
- Port conflicts on the metrics endpoint.
- Non-empty buffers after a process exits unexpectedly.

## Module 7: Generated Schema And DDL Ownership

Goal: understand where table definitions come from and how production DDL is
configured.

Topics:

- Concrete landing-table DDL is decoder-owned and renderer-generated.
- `carbon-core` provides schema execution and generic writer machinery, but it
  does not know Jupiter, Token Program, or any other program-specific schema.
- Generated table families follow these patterns:
  - `{program}_{instruction}_instruction_landing`
  - `{program}_cpi_event_landing`
  - `{program}_{account}_account_landing`
- The default DDL mode is local `MergeTree`.
- Renderer options can produce replicated or distributed table setups.
- Generated migrations are additive; destructive type-change migrations remain
  outside the sink.

Hands-on reading:

- `crates/core/src/clickhouse/docs/Carbon Sink Implementation.md`
- generated ClickHouse modules in the Jupiter Swap and Token Program decoder
  crates

Checkpoint questions:

- Why does the core runtime not infer table schemas from live rows?
- Why should known decoder-owned shapes stay typed instead of falling back to
  JSON by default?
- What operational problem do replicated or distributed DDL modes solve?

## Module 8: Production Boundaries

Goal: separate sink responsibilities from surrounding indexing system
responsibilities.

The ClickHouse sink handles:

- typed row conversion from decoder-owned schemas
- deterministic landing IDs
- local row and byte batching
- per-buffer flushing
- shutdown drain through processor finalization
- synchronous inserts and async-wait insert settings
- transient HTTP retry and backoff
- optional exact-batch insert deduplication tokens
- generated landing-table bootstrap
- sink-side metrics

ClickHouse handles:

- physical storage
- compression
- partitioning, sorting, and merges
- replicated and distributed topology
- server-side async insert batching
- materialized views and downstream serving tables
- retention and query-time performance

External control-plane or application code handles:

- durable queues
- durable retry state
- slot and range coverage tracking
- replay orchestration
- source checkpoints
- DLQs and poison-record policy
- Solana finality and reorg policy
- schema rollout coordination
- serving API semantics

Final checkpoint:

- Build a short deployment sketch for one backfill pipeline and one live
  ingestion pipeline.
- Identify which concerns are handled by Carbon, ClickHouse, and an external
  control plane.
- List the SQL and Prometheus checks that prove the smoke run completed cleanly.

