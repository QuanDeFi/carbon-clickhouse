# ClickHouse Sink Tutorial Curriculum

## Purpose

This curriculum documents the current Carbon ClickHouse sink tutorial recording.
The recording is a practical end-user walkthrough: verify the local stack, show
the example configuration, run both examples, observe the live pipelines, and
validate generated ClickHouse landing rows.

The current recording source of truth is `scripts/demo/runbook.yaml`. The
per-scene narration lives in `docs/tutorial-video/scene-*.md`.

## Examples

- `examples/jupiter-swap-clickhouse`
- `examples/token-program-clickhouse`

The Jupiter example demonstrates generated instruction and CPI/event landing
tables, plus TokenLedger account rows in pure live head-follow mode. The Token
Program example demonstrates fixed USDC account snapshots and live instruction
rows.

## Current Scene Files

Only these scene manuscripts are current:

- `docs/tutorial-video/scene-01-readiness-checks.md`
- `docs/tutorial-video/scene-02-example-env.md`
- `docs/tutorial-video/scene-03-jupiter-live.md`
- `docs/tutorial-video/scene-04-token-live.md`
- `docs/tutorial-video/scene-05-observability.md`
- `docs/tutorial-video/scene-06-clickhouse-play.md`

Any scene not listed here is not part of the final recording flow.

## Current Flow

1. Verify local readiness:
   ClickHouse, Prometheus, Grafana, and RPC reachability.
2. Show example environment files:
   database URL, RPC URL, metrics ports, log level, Jupiter slot range, and the
   async-wait insert toggle.
3. Start the Jupiter example:
   live mode, generated Jupiter table reconciliation, RPC block crawling,
   Jupiter decoding, and ClickHouse writes.
4. Start the Token Program example:
   generated table reconciliation, fixed USDC account snapshot, then live
   finalized block crawling for Token Program instructions.
5. Observe both examples:
   Grafana shows Carbon-side throughput, queue, latency, buffering, flush, and
   failure metrics. ClickStack shows ClickHouse-side insert rows and bytes per
   generated landing table.
6. Validate in ClickHouse Play:
   use the table browser, row/byte metadata, and representative decoded rows to
   prove that generated tables exist and data landed.

The examples keep running through the observability and ClickHouse Play scenes.
They are stopped after the recording, not inside the recording.

## Scene Timing Contract

The narration should drive scene duration. Do not compress wording just to fit
an older recording. If the script expands, update `runbook.yaml` timing and
record a new review video.

| Scene | Current minimum | Visual focus | Narration point |
| --- | ---: | --- | --- |
| 1. Readiness checks | 23s | terminal checks | Required services and RPC reachability are online before ingestion. |
| 2. Example env | 52s | both `.env.example` files | Users see where database, RPC, metrics, live-mode, and async-wait values live. |
| 3. Jupiter live | 39.3s | Jupiter terminal | Startup reconciles generated tables, connects to RPC, decodes live Jupiter activity, and writes rows. |
| 4. Token live | 29.8s | Token terminal | Startup writes fixed USDC account snapshots, then tails finalized Token Program instructions. |
| 5. Observability | 51.9s | Grafana and ClickStack | Grafana proves Carbon-side processing; ClickStack proves ClickHouse receives rows and bytes per table. |
| 6. ClickHouse Play | 70s | table browser, per-example table-family model, top populated tables, and largest table rows | Table metadata and decoded rows validate the pipeline end to end. |

## Scene Details

### 1. Readiness Checks

Show the required services are reachable before the examples start:

- In the demo terminal, `docker ps` shows only the three tutorial containers:
  ClickHouse, Prometheus, and Grafana.
- ClickHouse responds through its documented `/ping` endpoint.
- Prometheus readiness succeeds at `localhost:9090/-/ready`.
- Grafana health succeeds at `localhost:3000/api/health`.
- The configured RPC endpoint answers `getHealth`.

This is a readiness check, not proof that tutorial data has landed.

### 2. Example Environment

Show the example environment files, not private local values. Mention:

- `DATABASE_URL`: database URL.
- `RPC_URL`: RPC URL.
- `PROMETHEUS_METRICS_ADDR`: metrics endpoint.
- Jupiter slot range: empty start/end means pure live mode.
- `CLICKHOUSE_ASYNC_INSERT`: async-wait insert opt-in.

Sync inserts are the default. In this tutorial, async-wait inserts are enabled
because two example processes write small batches into the same ClickHouse
server. ClickHouse can group those writes server-side, while Carbon still waits
for acknowledgement.

Point users to `main.rs` in each example for env wiring, and to
`crates/core/src/clickhouse/config.rs` for the full sink config surface.

### 3. Jupiter Live

Jupiter starts with a plain `cargo run`. The env selects pure live mode and
async-wait inserts.

Startup should be explained in order:

- build ClickHouse configs
- reconcile generated instruction, event, and TokenLedger account tables
- derive a live start slot near finalized head
- connect to RPC through the block crawler
- decode Jupiter activity
- write generated landing rows while the process keeps running

### 4. Token Program Live

The Token Program example starts in a second terminal and uses a separate
metrics port.

Startup should be explained in order:

- reconcile generated account and instruction tables
- fetch four fixed USDC accounts with `getMultipleAccounts`
- write decoded account snapshot rows
- start the finalized block crawler from the next slot
- decode live Token Program instructions
- keep writing rows while Jupiter remains active

### 5. Observability

Grafana is the Carbon-side view:

- processed updates show throughput
- failed updates should stay quiet
- queue depth and processing latency show whether the pipeline keeps up
- buffered rows and bytes show pending ClickHouse work
- active buffers show current batching pressure
- inserted rows show successful sink output
- flush failures show sink-side write problems

The Token Program dashboard has the same layout for the second example. Do not
use wording that says we "confirm it briefly"; simply show that the same metrics
exist for the Token Program run.

ClickStack is the database-side view. Use the ClickHouse Dashboard Inserts tab,
not Chart Explorer and not query-log exploration. Show insert rows and bytes per
generated landing table. Do not discuss ClickHouse partitions or parts.

### 6. ClickHouse Play

ClickHouse Play is the final validation scene. Keep it focused on proof:

- generated landing tables exist
- row and byte metadata show which tables are populated
- the generated account, instruction, and CPI/event table families are visible
  per decoder/example
- the largest populated landing tables are visible by rows and bytes
- one high-volume decoded landing table is queryable

Use the current row-inspection target:

- `token_program_transfer_checked_instruction_landing`

Explain the data at a practical level:

- account tables store decoded account state
- instruction tables store decoded program instructions
- CPI/event tables store event-like activity emitted through inner instructions
- for a stablecoin such as USDC, checked transfers represent wallet-to-wallet
  sends, user deposits into protocols, or swap settlement movements

The menu and family summary explain the decoder-owned schema model. The largest
table's metadata plus decoded rows is the final end-to-end proof.

## Removed From The Current Recording

Do not add these back unless `scripts/demo/runbook.yaml` changes:

- setup script walkthroughs
- setup-file tour scenes
- architecture primer slides
- production-boundary slides
- async-insert evidence scene
- ClickStack Chart Explorer
- ClickStack query-log exploration
- ClickHouse partitions or parts explanation
- Grafana login or home page
- manual command variations during final recording
