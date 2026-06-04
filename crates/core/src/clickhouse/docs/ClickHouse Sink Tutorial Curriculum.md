# ClickHouse Sink Tutorial Curriculum

## Purpose

This curriculum teaches engineers how to use Carbon's ClickHouse sink to write
decoded Solana data into typed ClickHouse landing tables. It is organized as a
hands-on tutorial path, starting with the sink model and ending with local
validation, browser-based data inspection, real-time observability, and
direct ClickHouse row verification.

The current recording source of truth is `scripts/demo/runbook.yaml`. This
curriculum is the human-readable narrative plan for that runbook, and the
per-scene narration lives in `docs/tutorial-video/scene-*.md`.

The curriculum uses the current canary examples:

- `examples/jupiter-swap-clickhouse` for instruction and CPI/event landing
  rows, with TokenLedger account landing rows only in the live head-follow path.
- `examples/token-program-clickhouse` for fixed USDC account snapshots and live
  Token Program instruction landing rows.

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

Do not use the public mainnet-beta RPC endpoint for the examples. These smoke
tests need a production/provider RPC endpoint.

## Learning Outcomes

After completing the curriculum, learners should be able to:

- verify that ClickHouse, Prometheus, Grafana, and an RPC endpoint are reachable
  before ingestion
- identify the runtime inputs for database and RPC configuration
- find the core ClickHouse sink configuration surface in `carbon-core`
- run the Jupiter ClickHouse example with the configured slot range or live mode
- run the Token Program ClickHouse fixed-USDC account snapshot plus live
  instruction example
- identify the generated landing tables created by each example
- observe both live examples through Grafana and ClickStack
- inspect generated landing table contents through ClickHouse Play
- validate that ClickHouse config, RPC input, decoding, and writes all worked
- understand that insert modes, architecture, and production boundaries are
  deeper follow-up topics, not part of the current recording path

## Current Tutorial Recording Flow

The tutorial recording should feel like a guided live walkthrough, not a
smoke-test log capture. The primary path is: prove the required local services
are reachable, show the relevant example configuration in VS Code, run both
examples, observe the running pipelines, and verify generated rows in
ClickHouse.

The current six-scene flow is:

1. In a terminal, validate that the required local services are already online:
   ClickHouse, Prometheus, Grafana, and RPC reachability.
2. In VS Code, show both example `.env.example` files for database URL,
   RPC URL, slot range or live mode, metrics ports, log level, and the optional
   async-wait insert toggle.
   Mention that the full ClickHouse sink configuration reference lives in
   `crates/core/src/clickhouse/config.rs`, but keep the visual focus on the
   example environment files.
3. Start the Jupiter Swap ClickHouse example so it can decode Jupiter activity
   from the configured slot range or live path and write generated landing rows
   while the walkthrough continues.
4. Start the Token Program ClickHouse example in a second terminal so the fixed
   USDC account snapshot and live instruction tail run alongside Jupiter.
5. Show the filtered Jupiter and Token Program Grafana dashboards, then
   ClickStack, while both examples are active. Grafana is the Carbon-side
   pipeline view; ClickStack is the ClickHouse-side insert view.
6. Open ClickHouse Play at `/play`, use the table browser to show generated
   landing tables and table size metadata, then inspect representative Jupiter
   and Token Program rows. This is the final validation that the ClickHouse
   config was accepted, the RPC input was accepted, decoding happened,
   generated rows were written, and the rows are queryable.

Architecture slides, async-insert demonstrations, and production-boundary
slides are intentionally out of the current recording path. They remain useful
follow-up topics, but they should not be added back unless the runbook changes.

ClickHouse UI note:

- The current local tutorial stack should use ClickHouse `/play` for table
  inspection.
- ClickStack should be shown after both live examples are running. Use the
  embedded ClickHouse Dashboard's default Inserts tab to show insert activity
  per landing table from the database point of view.
- ClickHouse `/dashboards` is optional background context. The recording's
  primary observability comparison is Grafana for Carbon pipeline metrics plus
  ClickStack for ClickHouse-side insert activity.
- Richer third-party ClickHouse web explorers can be evaluated later, but they
  add installation, credentials, and recording complexity that is unnecessary
  for the first tutorial.

## Recording Scene And Narration Plan

Use this as the timing contract between the deterministic runbook and the
voiceover. The examples are intentionally not stopped after their terminal
scenes; Jupiter and Token Program keep running in parallel through the Grafana,
ClickStack, and ClickHouse Play validation scenes.

The latest validated no-audio run before this planning pass was about 3:15. The
tab-preloaded VS Code flow should keep the same overall shape while replacing
Explorer navigation time with narration time.

| Scene | Target visual budget | Subscene | Narration point |
| --- | ---: | --- | --- |
| 1. Local readiness checks | 21-23s | `docker compose ... ps` | The required local services are already online. |
|  |  | ClickHouse `SELECT 1` | ClickHouse accepts local HTTP queries before ingestion starts. |
|  |  | Prometheus and Grafana health checks | Metrics and dashboards are reachable before either example runs. |
|  |  | RPC `getHealth` | A public RPC status call proves network reachability without showing the private provider RPC URL used by the examples. |
| 2. Example environment files | 23-25s | Jupiter `.env.example` | Jupiter has the database URL, RPC URL, slot range, metrics port, log level, and async-wait toggle configured before the run command. |
|  |  | Token Program `.env.example` | Token Program uses its own metrics port so both examples can run together. |
|  |  | ClickHouse sink config reference | `crates/core/src/clickhouse/config.rs` is where users can find the full insert, batch, retry, transport, deduplication, source, mode, and decoder-version options. |
| 3. Jupiter example run | 21-23s | `cargo run ...` | Startup connects to RPC, decodes Jupiter activity from the env-file live path, and writes generated landing rows. |
| 4. Token Program live run | 24-26s | `cargo run ...` | The second pipeline starts beside Jupiter, writes the fixed USDC account snapshot, and tails live Token Program instructions. |
| 5. Observability | 25-27s | Jupiter Grafana dashboard | Jupiter is still running; Grafana shows Carbon-side processed updates and ClickHouse sink metrics for that pipeline. |
|  |  | Token Program Grafana dashboard | Token Program has its own filtered dashboard, proving both pipelines are active in parallel. |
|  |  | ClickStack Inserts tab | ClickStack shows ClickHouse-side insert activity per landing table. |
| 6. ClickHouse Play validation | 30-32s | generated table browser | Landing tables are visible with row and byte metadata. |
|  |  | representative Jupiter rows | Jupiter shows event and instruction landing rows; TokenLedger account rows appear only when the live head-follow path populates them. |
|  |  | representative Token Program rows | Token Program shows large instruction tables and account canary rows. |
|  |  | closing proof | The final proof combines accepted config, accepted RPC input, decoding, writes, and queryable landing rows. |

Voiceover should stay one idea per subscene. Do not read code line-by-line; use
the visible file or dashboard as evidence for the point being narrated.

The detailed modules below are broader curriculum material. The current video
recording uses the flow above.

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
- How exact-batch insert deduplication protects retry idempotency without
  turning landing tables into unique-key tables.
- How multiple Carbon processes write independently while ClickHouse observes
  insert attempts through query IDs, insert deduplication tokens, and row-level
  `source_name`, `mode`, and `decoder_version` metadata.
- Why the branch hardens the RPC block crawler for live ClickHouse examples:
  temporary near-head block errors are retried, permanent skipped slots are
  logged, downstream sends backpressure instead of dropping updates, and queued
  blocks drain on shutdown.

Hands-on reading:

- `crates/core/src/clickhouse/docs/ClickHouse Sink Architecture.md`
- `crates/core/src/clickhouse/docs/Carbon Sink Implementation.md`
- `crates/core/src/clickhouse/docs/Carbon Core v1 Architecture.md`

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
curl -fsSI 'http://carbon:carbon@localhost:8123/play' | sed -n '1,8p'
docker compose -f monitoring/compose.yaml up -d
docker compose -f monitoring/compose.yaml ps
```

Expected outcome:

- ClickHouse responds to `SELECT 1`.
- ClickHouse's built-in browser SQL UI is available at
  `http://localhost:8123/play`.
- Prometheus is available at `http://localhost:9090`.
- Grafana is available at `http://localhost:3000`.
- Grafana provisions the combined overview plus filtered Jupiter and Token
  Program dashboards from `monitoring/grafana/dashboards/`.
- The Jupiter example can expose metrics at `0.0.0.0:9464/metrics`.
- The Token Program example can expose metrics at `0.0.0.0:9465/metrics`.

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
  the current finalized slot using `BLOCK_CRAWLER_HEAD_LAG_SLOTS`.

Teaching notes:

- The example bootstraps generated Jupiter instruction landing tables.
- It also bootstraps generated Jupiter CPI/event landing tables.
- For the tutorial recording, use pure head-follow mode instead of a bounded
  backfill. The live run keeps producing Carbon metrics and ClickHouse insert
  activity while the tutorial moves through Grafana, ClickStack, and `/play`.
- Keep the Jupiter terminal open while the Token Program, observability, and
  ClickHouse Play scenes run.
- In pure head-follow mode, it attaches the generated TokenLedger account
  processor and fetches current confirmed TokenLedger account snapshots for
  first-seen TokenLedger pubkeys.
- TokenLedger account fetching is live-only because normal RPC account reads
  return current confirmed account state, not historical state for an old
  backfill slot.
- Pure head-follow mode therefore bootstraps one additional Jupiter account
  table, `jupiter_swap_token_ledger_account_landing`. Bounded backfills do not
  create that account table through this example.
- Empty instruction tables are expected when the selected slot window does not
  contain that instruction type.

Validation queries:

```sql
SHOW TABLES FROM default LIKE 'jupiter_swap_%landing';

SELECT 'fee_event' AS event_table, count()
FROM default.jupiter_swap_fee_event_landing
UNION ALL
SELECT 'swap_event' AS event_table, count()
FROM default.jupiter_swap_swap_event_landing
UNION ALL
SELECT 'swaps_event' AS event_table, count()
FROM default.jupiter_swap_swaps_event_landing
UNION ALL
SELECT 'candidate_swap_results' AS event_table, count()
FROM default.jupiter_swap_candidate_swap_results_landing
UNION ALL
SELECT 'candidate_swap_quote_error' AS event_table, count()
FROM default.jupiter_swap_candidate_swap_quote_error_landing
UNION ALL
SELECT 'best_swap_out_amount_violation' AS event_table, count()
FROM default.jupiter_swap_best_swap_out_amount_violation_landing;

SELECT
  count() AS rows,
  uniq(signature) AS signatures
FROM default.jupiter_swap_route_instruction_landing;
```

Pure head-follow mode also enables the TokenLedger account path:

```sql
SELECT
  count() AS token_ledger_snapshots,
  uniq(pubkey) AS token_ledger_accounts
FROM default.jupiter_swap_token_ledger_account_landing;
```

Expected outcome:

- At least one Jupiter landing table exists.
- The generated CPI/event landing tables exist, including fee, swap, grouped
  swap, candidate quote, quote error, and output-violation event families.
- In pure head-follow mode, the TokenLedger account table exists and may receive
  current account snapshots for first-seen TokenLedger pubkeys.
- If the selected blocks include decoded Jupiter activity, row counts increase.
- Sink buffers drain on shutdown.

## Module 4: Token Program Account And Instruction Ingestion

Goal: fetch real USDC account snapshots and then keep tailing finalized blocks
so generated Token Program instruction landing tables receive live rows.

Example directory:

```sh
cd examples/token-program-clickhouse
```

Environment:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=<provider-rpc-url>
PROMETHEUS_METRICS_ADDR=0.0.0.0:9465
LOG_LEVEL=info
```

Run the fixed USDC snapshot plus live instruction tail:

```sh
cargo run -p token-program-clickhouse-carbon-example
```

Teaching notes:

- The example uses standard Solana JSON-RPC `getMultipleAccounts` against four
  hardwired USDC accounts: the USDC mint, the USDC mint authority multisig, the
  USDC freeze authority multisig, and one USDC token holding account.
- This intentionally avoids unbounded Token Program scans and provider-specific
  GPA modes.
- It populates generated mint, multisig, and token account landing tables from
  the fixed snapshot before starting the live instruction phase.
- Account rows are current account snapshots fetched at the RPC response slot,
  so the example writes `mode = snapshot` and
  `source_name = rpc_get_multiple_accounts`.
- Snapshot rows have `transaction_signature = NULL` because they are current
  account reads, not transaction-scoped account updates.
- After the snapshot, the example starts the RPC block crawler at the next slot
  and decodes live Token Program instructions from finalized blocks.
- Instruction rows use `mode = live` and `source_name = rpc_block_crawler`.
- The process keeps running until interrupted, so Prometheus can scrape both
  account and instruction metrics at `PROMETHEUS_METRICS_ADDR`.
- For the tutorial recording, start this example in a second terminal window
  after Jupiter is already running. Keep both examples active while Grafana and
  ClickStack are shown and while ClickHouse Play verifies generated rows.

Validation queries:

```sql
SHOW TABLES FROM default LIKE 'token_program_%landing';

SELECT
  name,
  total_rows
FROM system.tables
WHERE database = 'default'
  AND name LIKE 'token_program_%landing'
ORDER BY name;

SELECT
  count() AS rows,
  uniq(pubkey) AS accounts,
  min(slot) AS min_slot,
  max(slot) AS max_slot
FROM default.token_program_token_account_landing;

SELECT
  count() AS rows,
  uniq(signature) AS signatures,
  min(slot) AS min_slot,
  max(slot) AS max_slot
FROM default.token_program_transfer_checked_instruction_landing;
```

Expected outcome:

- The Token Program account landing tables exist.
- `token_program_mint_account_landing` contains the USDC mint snapshot.
- `token_program_multisig_account_landing` contains the USDC mint and freeze
  authority multisig snapshots.
- `token_program_token_account_landing` contains the fixed USDC token holding
  account snapshot.
- The generated Token Program instruction landing tables exist.
- High-volume live tables such as `transfer_checked`, `initialize_account3`,
  `initialize_immutable_owner`, `get_account_data_size`, `close_account`, and
  `sync_native` may receive rows depending on current chain activity.
- Low-frequency instruction tables can remain empty in short runs.

## Module 5: Browser-Based ClickHouse Data Inspection

Goal: inspect real generated landing table contents in a ClickHouse-specific
browser UI.

Browser UI:

```text
http://localhost:8123/play
```

Optional ClickHouse-native UI:

```text
http://localhost:8123/dashboards
http://localhost:8123/clickstack
```

Teaching notes:

- Use ClickHouse's built-in `/play` UI for the tutorial. It is specific to
  ClickHouse, already served by the local ClickHouse HTTP port, and does not
  require a separate database explorer container.
- Use `/dashboards` only as a short ClickHouse server-health aside if the video
  needs database-level charts such as query rate, CPU, merges, and selected
  bytes.
- Use `/clickstack` for ClickHouse-side observability. In this tutorial, use
  the embedded ClickHouse Dashboard's Inserts tab so it complements Grafana by
  showing ClickHouse-side insert activity per landing table while the live
  examples are running.
- Use the browser scene for table inspection, not for secret configuration. Do
  not show provider RPC URLs or non-local credentials.
- The viewer should see table lists, row and byte metadata, and short sample
  rows. Table metadata proves ingestion happened; sample rows explain what
  actually landed.
- Prefer ClickHouse Play's built-in table browser and default `SELECT * ...
  LIMIT 100` behavior before adding custom SQL.

Useful Jupiter inspection queries:

```sql
SHOW TABLES FROM default LIKE 'jupiter_swap_%landing';

SELECT
  count() AS rows,
  uniq(signature) AS signatures
FROM default.jupiter_swap_route_instruction_landing;

SELECT
  slot,
  left(signature, 12) AS signature_prefix,
  in_amount,
  quoted_out_amount,
  length(route_plan) AS route_legs
FROM default.jupiter_swap_route_instruction_landing
LIMIT 5;

SELECT
  amm,
  input_amount,
  output_amount
FROM default.jupiter_swap_swap_event_landing
LIMIT 5;
```

Useful Token Program inspection queries:

```sql
SHOW TABLES FROM default LIKE 'token_program_%landing';

SELECT
  name,
  total_rows
FROM system.tables
WHERE database = 'default'
  AND name LIKE 'token_program_%landing'
ORDER BY name;

SELECT
  left(pubkey, 12) AS account_prefix,
  left(mint, 12) AS mint_prefix,
  amount,
  state
FROM default.token_program_token_account_landing
LIMIT 5;

SELECT
  count() AS rows,
  uniq(signature) AS signatures,
  min(slot) AS min_slot,
  max(slot) AS max_slot
FROM default.token_program_transfer_checked_instruction_landing;
```

Expected outcome:

- The viewer can see the actual ClickHouse landing tables, not just terminal
  wrapper output.
- Jupiter route and event rows are understandable from sample fields such as
  `in_amount`, `quoted_out_amount`, route-leg count, AMM, input amount, and
  output amount.
- Token Program account rows are understandable as current USDC account
  snapshots, while instruction rows are transaction-scoped live decoded
  instructions.

## Module 6: Insert Modes And Runtime Configuration

Goal: understand how the sink sends batches to ClickHouse.

Topics:

- Synchronous inserts are the default for deterministic backfills and bounded
  crawls.
- Async-wait inserts are intended for live multi-writer deployments where
  ClickHouse should coalesce inserts server-side while Carbon still waits for
  acknowledgement.
- Async mode always waits for ClickHouse acknowledgement with
  `wait_for_async_insert=1`.
- Batch settings control local row and byte buffering.
- Retry settings handle transient HTTP failures.
- Transport settings control request timeout, connection timeout, pooling,
  compression, and user agent.
- Exact-batch deduplication tokens are enabled by default for retry
  idempotency. They can be disabled by callers that explicitly accept duplicate
  retry inserts in raw landing tables.
- Per-attempt query IDs are generated for ClickHouse query-log and async-log
  traceability.
- `source_name`, `mode`, and `decoder_version` are row metadata; use distinct
  source names when multiple production pipelines need to be separated in SQL.
- Each Carbon process owns local buffers, and ClickHouse receives inserts with
  per-attempt query IDs, exact-batch deduplication tokens, optional async-wait
  settings, and row-level source metadata.
- Generated local `MergeTree` landing tables include
  `non_replicated_deduplication_window = 1000` by default so those tokens are
  effective for plain local smoke-test tables.

Jupiter async-wait run:

```sh
CLICKHOUSE_ASYNC_INSERT=true cargo run -p jupiter-swap-clickhouse-carbon-example
```

Token Program async-wait run:

```sh
CLICKHOUSE_ASYNC_INSERT=true cargo run -p token-program-clickhouse-carbon-example
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
- What identifies a Carbon insert attempt in ClickHouse logs?
- Which metadata should be used to separate multiple Carbon processes in SQL?
- What should happen to buffered rows during shutdown?

## Module 7: Observability And Health Checks

Goal: validate that the Carbon process and ClickHouse sink are healthy while
examples are actively running.

Start monitoring:

```sh
docker compose -f monitoring/compose.yaml up -d
```

Inspect the Carbon metrics endpoint:

```sh
curl -sS http://localhost:9464/metrics | rg 'carbon_updates_failed_total|clickhouse_'
curl -sS http://localhost:9465/metrics | rg 'clickhouse_(accounts|instructions)_'
```

The Jupiter example exposes Prometheus metrics on `9464` by default. The Token
Program example exposes ClickHouse account and instruction metrics on `9465` by
default, so both examples can be monitored locally without a port conflict.

Grafana dashboard:

```text
http://localhost:3000
```

Use the provisioned `Carbon ClickHouse Overview` dashboard for the tutorial.
Show it while an example is actively ingesting data, not only after the process
has already exited.

Important panels to explain:

- `Carbon Updates`: processed and failed update rates.
- `Carbon Queue Depth`: whether the process is falling behind.
- `Carbon Processing Latency`: processor timing.
- `ClickHouse Buffered Rows`: local rows currently waiting to flush.
- `ClickHouse Buffered Bytes`: local buffered payload size.
- `ClickHouse Active Buffers`: how many table/partition buffers are open.
- `ClickHouse Inserted Rows`: ClickHouse sink insert throughput.
- `ClickHouse Errors And Retries`: transient or permanent write pressure.
- `ClickHouse Backpressure Rejections`: rows rejected after local drain fails.

Prometheus queries:

```promql
carbon_updates_queued
rate(carbon_updates_processed_total[1m])
rate(carbon_updates_failed_total[1m])
increase(carbon_updates_processed_total[10m])
increase(clickhouse_instructions_inserted[10m])
clickhouse_instructions_buffered_rows
rate(clickhouse_instructions_flush_failed_batches[1m])
clickhouse_accounts_buffered_rows
rate(clickhouse_accounts_flush_failed_batches[1m])
```

Healthy signals:

- `carbon_updates_failed_total` stays at zero for the smoke run.
- `increase(carbon_updates_processed_total[10m])` is positive during or shortly
  after the run.
- `increase(clickhouse_instructions_inserted[10m])` is positive after decoded
  Jupiter or Token Program instruction rows insert.
- `clickhouse_instructions_flush_failed_batches` stays at zero for Jupiter.
- `clickhouse_accounts_flush_failed_batches` stays at zero for account rows.
- `clickhouse_instructions_flush_failed_batches` stays at zero for Token
  Program instruction rows.
- Buffered row gauges return to zero after finite shutdown.
- Retry rates do not grow continuously.

Troubleshooting focus:

- RPC provider timeouts or unsupported methods.
- Missing or malformed `.env` values.
- ClickHouse authentication or endpoint errors.
- Port conflicts on the metrics endpoint.
- Non-empty buffers after a process exits unexpectedly.

## Module 8: Generated Schema And DDL Ownership

Goal: understand where table definitions come from and how production DDL is
configured.

Topics:

- Concrete landing-table DDL is decoder-owned and renderer-generated.
- `carbon-core` provides schema execution and generic writer machinery, but it
  does not know Jupiter, Token Program, or any other program-specific schema.
- Generated table families follow these patterns:
  - `{program}_{instruction}_instruction_landing`
  - `{program}_{event}_landing`
  - `{program}_{account}_account_landing`
- The default DDL mode is local `MergeTree`.
- Renderer options can produce replicated or distributed table setups.
- The renderer CLI accepts `--with-clickhouse true` for defaults and
  `--clickhouse-options <json-or-file>` for production DDL options.
- Generated managed schema metadata is the source of truth for table shape.
- Schema reconciliation creates missing tables, adds missing columns, validates
  table layout, safely extends enum columns, and fails fast on unsafe drift.
- Destructive type-change migrations and table recreation remain outside the
  ingestion startup path.
- `scripts/validate-clickhouse-decoder-rollout.sh --clickhouse-ddl-smoke` can
  bootstrap the committed canary schemas in a temporary ClickHouse database and
  verify generated event table sort keys.

Hands-on reading:

- `crates/core/src/clickhouse/docs/Carbon Sink Implementation.md`
- generated ClickHouse modules in the Jupiter Swap and Token Program decoder
  crates

Checkpoint questions:

- Why does the core runtime not infer table schemas from live rows?
- Why should known decoder-owned shapes stay typed instead of falling back to
  JSON by default?
- What operational problem do replicated or distributed DDL modes solve?

## Module 9: Production Boundaries

Goal: separate sink responsibilities from surrounding indexing system
responsibilities.

The ClickHouse sink handles:

- typed row conversion from decoder-owned schemas
- deterministic landing IDs
- local row and byte batching
- per-buffer flushing
- shutdown drain through processor finalization
- synchronous inserts and async-wait insert settings
- per-attempt query IDs for ClickHouse observability
- transient HTTP retry and backoff
- default exact-batch insert deduplication tokens
- generated managed schema bootstrap and safe drift reconciliation
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
- multi-process orchestration and scrape-target labeling
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
