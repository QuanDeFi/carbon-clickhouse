## Carbon ClickHouse Sink

`clickhouse-upstream-v1` a minimal ClickHouse sink implementation

## What this branch adds

This branch does **not** add a general-purpose ClickHouse storage subsystem to Carbon. It adds a **minimal but real ClickHouse landing sink** that fits Carbon’s existing processor architecture.

Concretely, it adds four things:

1. a small generic ClickHouse runtime inside `carbon-core`
2. a core lifecycle change so buffered processors can flush on shutdown
3. one concrete decoder-owned ClickHouse implementation for **Jupiter swap CPI `SwapEvent`**
4. enough codegen support to keep this from being a pure one-off integration   


## How it fits into Carbon

ClickHouse is implemented as a **normal Carbon instruction processor**, not as a new pipeline type and not as an out-of-band sink framework.

The runtime shape is:

1. a datasource emits Solana transaction updates
2. Carbon builds decoded instruction inputs
3. a ClickHouse-backed instruction processor receives those decoded inputs
4. decoder-owned code maps them into ClickHouse row types
5. a generic ClickHouse batch writer serializes and inserts them into ClickHouse over HTTP  

That choice keeps the ClickHouse path aligned with Carbon’s existing model:

* pipelines route data
* processors own side effects
* decoder crates own schema and storage-specific row mapping

So this is **Carbon-native**, not a sidecar design.


## The core architectural change: processor finalization

The most important Carbon-core change is **not** the HTTP client or the table DDL. It is the addition of **processor finalization**.

Because ClickHouse writes are buffered, accepted rows may still be sitting in memory when the pipeline shuts down. Upstream Carbon did not previously have a formal processor shutdown hook. This branch adds one:

* `Processor::finalize()`
* pipe wrappers forward `finalize()`
* `Pipeline` calls `finalize_pipes()` on shutdown paths before exporter shutdown  

That means the ClickHouse sink can flush buffered rows deterministically on:

* datasource cancellation
* Ctrl-C with immediate shutdown
* channel closure

Without this, buffered rows could be lost. This is the main correctness improvement introduced by the ClickHouse port.


## What lives in `carbon-core`

The generic ClickHouse runtime in `carbon-core` is intentionally small:

* `config.rs` — endpoint, database, auth, table name, source metadata, batch settings
* `http.rs` — minimal HTTP transport over `reqwest`
* `admin.rs` — explicit schema/bootstrap execution
* `rows/mod.rs` — row/table traits and deterministic event ID helper
* `writer.rs` — buffered batch writer
* `processors.rs` — ClickHouse instruction processor and sink metrics       

Important boundary:

* core knows **how to talk to ClickHouse**
* core does **not** know any Jupiter-specific schema details

That split is deliberate, since decoders own the program specific data schema.


## Write path and insert model

The sink uses **client-side batching** and **HTTP `JSONEachRow` inserts**.

Current behavior:

* rows are buffered in memory
* buffers are grouped by `partition_key()`
* flush happens when `max_rows` is reached
* flush also happens when `flush_interval` has elapsed
* rows are serialized with `serde::Serialize`
* inserts go through ClickHouse HTTP, not the native protocol   

Important nuance: `flush_interval` is **not** driven by a background timer thread. It is checked when a new row arrives, and any remaining buffer is also drained in `finalize()`. So the implementation is **opportunistic interval flushing**, not autonomous periodic flushing. That matters for evaluating latency and idle-buffer behavior. 

The design choice is optimized for a minimal, predictable landing sink


## Current scope

The current implemented scope is narrow:

* **instruction / CPI-event ingestion only**
* **one decoder family**: Jupiter swap
* **one concrete event path**: `CpiEvent::SwapEvent`
* **one landing table family**: `jupiter_swap_swap_event_landing`   

The decoder-owned layer does the program-specific work:

* define the typed row
* define table DDL
* map decoded event data into rows
* bootstrap the schema
* provide sink defaults and setup helpers  

So this branch proves:

* Carbon can decode real Solana data
* Carbon can convert it into typed analytical rows
* Carbon can batch and land those rows in ClickHouse correctly enough for real use

It does **not** prove broad ClickHouse coverage across the Carbon repo.


## Identity, replay, and table semantics

The current sink is **landing-only** and **append-only**.

It uses a deterministic event identifier built from:

* program id
* transaction signature
* instruction absolute path
* event type
* event sequence  

That gives stable row identity across reprocessing, which is useful for future canonicalization or dedup logic.

But the sink does **not** currently do:

* online deduplication
* serving-table resolution
* replay convergence
* coverage/range tracking
* durable retry state

So the present contract is:

* **stable landing identity**
* **not full canonical warehouse semantics**

This is an important limitation and should be read as intentional, not accidental.


## Metrics and observability

The sink adds its own metrics because batch sinks need different visibility than row-at-a-time sinks.

Tracked today:

* inserted rows
* failed rows in failed batches
* current buffered row count
* successful flush batches
* failed flush batches
* flush duration histogram  

That is enough for a first implementation, but still limited compared with a fuller ingestion service. There is no deeper range/coverage bookkeeping yet.


## Generator state

This is not purely handwritten. The branch also adds partial ClickHouse generator support:

* generated decoder manifest support for a `clickhouse` feature
* generated ClickHouse instruction glue
* generated generic CPI-event landing-row template
* generated decode-path changes so CPI events can be handled explicitly before generic instruction macro dispatch     

But the generator support is still **partial**. The real end-to-end proven path is still the concrete Jupiter implementation.

---

