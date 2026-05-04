## Branch diff: `upstream-v1-sync` to `clickhouse-upstream-v1`

This document summarizes the current ClickHouse branch relative to
`upstream-v1-sync`.

## Overall shape

Branch relation before this documentation refresh:

- `clickhouse-upstream-v1` is ahead of `upstream-v1-sync` by 12 commits.
- `clickhouse-upstream-v1` is behind `upstream-v1-sync` by 0 commits.
- `upstream-v1-sync` is an ancestor of `clickhouse-upstream-v1`.

File counts from `git diff --name-status upstream-v1-sync...HEAD`:

- Added files: 26
- Modified files: 23
- Removed files: 0
- Total changed files: 49

Patch size from `git diff --stat upstream-v1-sync...HEAD` before this refresh:

- 6,639 insertions
- 16 deletions

Core meaning of the branch:

- Adds a ClickHouse landing-table backend in `carbon-core`.
- Adds processor finalization so buffered sinks can drain on shutdown.
- Adds generated typed landing rows for instruction, CPI-event, and account families.
- Adds Jupiter swap instruction and CPI-event ClickHouse canary coverage.
- Adds Token Program account ClickHouse canary coverage.
- Adds a bounded Jupiter swap ClickHouse example for real RPC-to-ClickHouse testing.
- Hardens the writer for production-style multi-writer ingestion with per-buffer flushing and optional async-wait inserts.
- Documents the current landing-table architecture and near-term production boundaries.

## Touched path tree

```text
[M] .gitignore                         - local ClickHouse/env cleanup
[M] Cargo.lock                         - lockfile dependency state
[M] Cargo.toml                         - workspace dependency additions

crates/
├── core/
│   ├── [M] Cargo.toml                 - clickhouse feature and optional deps
│   └── src/
│       ├── [M] account.rs             - account pipe finalization
│       ├── [M] account_deletion.rs    - account-deletion pipe finalization
│       ├── [M] block_details.rs       - block pipe finalization
│       ├── [M] instruction.rs         - instruction pipe finalization
│       ├── [M] lib.rs                 - clickhouse module export
│       ├── [M] pipeline.rs            - shutdown finalization calls
│       ├── [M] processor.rs           - Processor::finalize()
│       ├── [M] transaction.rs         - transaction pipe finalization
│       └── clickhouse/
│           ├── [A] admin.rs           - schema/admin query executor
│           ├── [A] config.rs          - sink config and insert settings
│           ├── [A] http.rs            - HTTP transport helpers
│           ├── [A] metrics.rs         - shared instruction/account metrics
│           ├── [A] mod.rs             - module barrel and public exports
│           ├── [A] processors.rs      - instruction/account processors
│           ├── [A] writer.rs          - per-buffer batch writer and worker
│           ├── docs/
│           │   ├── [A] architecture.md
│           │   ├── [A] Branch Diff to v1.md
│           │   └── [A] Carbon Sink Implementation.md
│           └── rows/
│               └── [A] mod.rs         - row/table traits and stable IDs

decoders/
├── jupiter-swap-decoder/
│   ├── [M] Cargo.toml
│   └── src/instructions/
│       ├── [M] cpi_event.rs
│       ├── [M] mod.rs
│       └── clickhouse/
│           ├── [A] cpi_event_row.rs
│           ├── [A] instruction_rows.rs
│           └── [A] mod.rs
└── token-program-decoder/
    ├── [M] Cargo.toml
    └── src/accounts/
        ├── [M] mod.rs
        └── clickhouse/
            ├── [A] mint_row.rs
            ├── [A] mod.rs
            ├── [A] multisig_row.rs
            └── [A] token_row.rs

examples/
└── jupiter-swap-clickhouse/
    ├── [A] Cargo.toml
    └── src/
        └── [A] main.rs

packages/
└── renderer/
    ├── [M] package.json
    ├── src/
    │   ├── [M] cargoTomlGenerator.ts
    │   ├── [A] clickhouseRowMapper.ts
    │   └── [M] getRenderMapVisitor.ts
    ├── templates/
    │   ├── [A] accountsClickHouseMod.njk
    │   ├── [M] accountsMod.njk
    │   ├── [A] clickhouseRowPage.njk
    │   ├── [A] eventInstructionClickHouseRowPage.njk
    │   ├── [M] eventInstructionPage.njk
    │   ├── [A] instructionsClickHouseMod.njk
    │   └── [M] instructionsMod.njk
    └── test/
        └── [A] clickhouse-renderer.test.cjs
```

Legend:

- `[A]` added
- `[M]` modified

## Consolidated file-by-file summary

### Workspace and manifests

| File | Patch size | Summary |
| --- | ---: | --- |
| `.gitignore` | `+3 / -1` | Ignores local ClickHouse/runtime artifacts used during local testing. |
| `Cargo.lock` | `+25 / -0` | Pins the dependency graph implied by the ClickHouse feature and HTTP transport. |
| `Cargo.toml` | `+1 / -0` | Adds workspace-level `reqwest` for ClickHouse HTTP transport. |
| `crates/core/Cargo.toml` | `+4 / -0` | Adds the `clickhouse` feature and optional dependencies. |
| `decoders/jupiter-swap-decoder/Cargo.toml` | `+11 / -0` | Adds generated ClickHouse feature/dependencies for Jupiter. |
| `decoders/token-program-decoder/Cargo.toml` | `+7 / -0` | Adds generated ClickHouse feature/dependencies for Token Program accounts. |
| `examples/jupiter-swap-clickhouse/Cargo.toml` | `+18 / -0` | Adds the example crate manifest. |
| `packages/renderer/package.json` | `+2 / -1` | Adds/updates renderer test wiring for ClickHouse generation tests. |

### Core lifecycle changes

| File | Patch size | Summary |
| --- | ---: | --- |
| `crates/core/src/processor.rs` | `+4 / -0` | Adds default `Processor::finalize()`, preserving compatibility for existing processors. |
| `crates/core/src/pipeline.rs` | `+27 / -0` | Calls pipe finalization on shutdown paths before exporter shutdown. |
| `crates/core/src/account.rs` | `+6 / -1` | Forwards finalization to account processors. |
| `crates/core/src/account_deletion.rs` | `+6 / -1` | Forwards finalization to account-deletion processors. |
| `crates/core/src/block_details.rs` | `+6 / -1` | Forwards finalization to block-details processors. |
| `crates/core/src/instruction.rs` | `+5 / -0` | Forwards finalization to instruction processors. |
| `crates/core/src/transaction.rs` | `+6 / -1` | Forwards finalization to transaction processors. |
| `crates/core/src/lib.rs` | `+2 / -0` | Exposes `carbon_core::clickhouse` behind the `clickhouse` feature. |

The lifecycle changes are intentionally small. They exist because ClickHouse is buffered: once a processor accepts a row, the pipeline needs a finalization hook to drain in-memory buffers before process exit.

### Core ClickHouse runtime

| File | Patch size | Summary |
| --- | ---: | --- |
| `crates/core/src/clickhouse/admin.rs` | `+50 / -0` | Adds `ClickHouseSchema` and `ClickHouseAdmin` for explicit schema/bootstrap execution. |
| `crates/core/src/clickhouse/config.rs` | `+245 / -0` | Adds endpoint/database/auth/batching config, row context, URL parsing, and typed insert settings. |
| `crates/core/src/clickhouse/http.rs` | `+78 / -0` | Adds HTTP query and insert helpers with shared ClickHouse query settings. |
| `crates/core/src/clickhouse/metrics.rs` | `+163 / -0` | Adds shared metrics for instruction and account ClickHouse families. |
| `crates/core/src/clickhouse/mod.rs` | `+13 / -0` | Re-exports the ClickHouse runtime API. |
| `crates/core/src/clickhouse/processors.rs` | `+400 / -0` | Adds `ClickHouseInstructionProcessor` and `ClickHouseAccountProcessor`, including shutdown drain behavior. |
| `crates/core/src/clickhouse/rows/mod.rs` | `+109 / -0` | Adds row/table traits, row context, multi-row contract, and deterministic instruction/event/account IDs. |
| `crates/core/src/clickhouse/writer.rs` | `+841 / -0` | Adds the buffered writer, keyed per `(table, partition)`, with per-buffer flushes, background stale-buffer flushing, all-buffer drain, shutdown, reinsertion on failed batches, and metric reporting. |

Important writer behavior now implemented:

- Rows are buffered by `(table, partition_key())`.
- A hot buffer reaching `max_rows` flushes only that buffer.
- The background worker flushes only buffers whose own `last_flush` exceeds `flush_interval`.
- `flush()` drains all buffers.
- `shutdown()` drains all buffers and stops the background worker.
- Failed flushes reinsert rows into the same buffer and surface errors on later operations or shutdown.
- Sync inserts are the default.
- Optional async mode always emits `async_insert=1` and `wait_for_async_insert=1`; no public fire-and-forget mode is exposed.

### Decoder canaries

| File | Patch size | Summary |
| --- | ---: | --- |
| `decoders/jupiter-swap-decoder/src/instructions/cpi_event.rs` | `+19 / -0` | Adds CPI-event account construction needed by generated event ClickHouse mapping. |
| `decoders/jupiter-swap-decoder/src/instructions/mod.rs` | `+15 / -1` | Exposes the ClickHouse module and decodes CPI events through the explicit pre-check path. |
| `decoders/jupiter-swap-decoder/src/instructions/clickhouse/cpi_event_row.rs` | `+521 / -0` | Adds typed Jupiter CPI-event landing rows and DDL for all generated event variants. |
| `decoders/jupiter-swap-decoder/src/instructions/clickhouse/instruction_rows.rs` | `+570 / -0` | Adds typed Jupiter instruction landing rows and DDL for all generated instruction variants. |
| `decoders/jupiter-swap-decoder/src/instructions/clickhouse/mod.rs` | `+536 / -0` | Adds Jupiter ClickHouse row enum, migrations, setup helpers, and dispatch across instruction and event row families. |
| `decoders/token-program-decoder/src/accounts/mod.rs` | `+3 / -0` | Exposes the generated account ClickHouse module behind the feature flag. |
| `decoders/token-program-decoder/src/accounts/clickhouse/mint_row.rs` | `+156 / -0` | Adds typed Token Program mint account landing row and DDL. |
| `decoders/token-program-decoder/src/accounts/clickhouse/multisig_row.rs` | `+156 / -0` | Adds typed Token Program multisig account landing row and DDL. |
| `decoders/token-program-decoder/src/accounts/clickhouse/token_row.rs` | `+168 / -0` | Adds typed Token Program token account landing row and DDL. |
| `decoders/token-program-decoder/src/accounts/clickhouse/mod.rs` | `+172 / -0` | Adds Token Program account row enum, migrations, setup helpers, and account processor wiring. |

The canary coverage is now broader than the initial handwritten Jupiter CPI-event path:

- Jupiter instruction rows cover all generated Jupiter instruction variants.
- Jupiter CPI-event rows cover all generated Jupiter event variants.
- Token Program account rows cover mint, token, and multisig account variants.

### Example

| File | Patch size | Summary |
| --- | ---: | --- |
| `examples/jupiter-swap-clickhouse/src/main.rs` | `+97 / -0` | Adds an end-to-end bounded backfill pipeline from RPC blocks into Jupiter ClickHouse landing tables. |

The example validates the real path:

1. Load `RPC_URL` and `DATABASE_URL`.
2. Run generated Jupiter ClickHouse schema bootstrap.
3. Crawl a bounded slot range with `RpcBlockCrawler`.
4. Decode Jupiter instructions and CPI events.
5. Write generated typed rows to ClickHouse through `ClickHouseInstructionProcessor`.
6. Drain buffered rows during pipeline shutdown.

It does not validate account rows or async insert mode by itself. Token Program account coverage is validated separately by compile/tests and needs an account-focused pipeline for a real-world data test.

### Renderer and generator

| File | Patch size | Summary |
| --- | ---: | --- |
| `packages/renderer/src/cargoTomlGenerator.ts` | `+21 / -2` | Adds `withClickHouse` manifest feature generation and required ClickHouse dependencies. |
| `packages/renderer/src/clickhouseRowMapper.ts` | `+246 / -0` | Adds ClickHouse field mapping for Rust fields, ClickHouse column types, expressions, and imports. |
| `packages/renderer/src/getRenderMapVisitor.ts` | `+72 / -1` | Adds conditional generation for account, instruction, and event ClickHouse modules. |
| `packages/renderer/templates/accountsMod.njk` | `+7 / -1` | Emits the account ClickHouse module behind `#[cfg(feature = "clickhouse")]`. |
| `packages/renderer/templates/accountsClickHouseMod.njk` | `+152 / -0` | Generates account row enum dispatch, migrations, setup helpers, and account processor wiring. |
| `packages/renderer/templates/clickhouseRowPage.njk` | `+266 / -0` | Generates typed account/instruction landing rows with stable IDs and DDL. |
| `packages/renderer/templates/eventInstructionClickHouseRowPage.njk` | `+163 / -0` | Generates typed per-event CPI landing rows with stable event IDs and DDL. |
| `packages/renderer/templates/eventInstructionPage.njk` | `+18 / -1` | Adds generated CPI-event account construction support. |
| `packages/renderer/templates/instructionsClickHouseMod.njk` | `+206 / -0` | Generates instruction/event row enum dispatch, migrations, setup helpers, and processor wiring. |
| `packages/renderer/templates/instructionsMod.njk` | `+24 / -4` | Emits the instruction ClickHouse module and explicit CPI-event pre-check path. |
| `packages/renderer/test/clickhouse-renderer.test.cjs` | `+132 / -0` | Adds renderer tests for ClickHouse modules, row generation, migrations, and dependencies. |

This turns ClickHouse from a handwritten Jupiter-only sink into a generator-backed target. The repository is still canary-limited; it has not regenerated every decoder with ClickHouse output.

## Current interpretation

This branch is best understood as four layers:

1. Core lifecycle support: `Processor::finalize()`, pipe forwarding, and pipeline shutdown calls.
2. Core ClickHouse runtime: config, HTTP, schema execution, row contracts, metrics, processors, and buffered writer.
3. Generated canary decoders: Jupiter instructions/events and Token Program accounts.
4. Renderer support: feature-gated generator templates for future decoder-wide ClickHouse rollout.

The branch intentionally remains landing-table-only. It does not implement serving/canonicalization tables, coverage/range tracking, online row deduplication, durable retry queues, or replicated/distributed DDL generation.

## Validation performed during this implementation

The following checks were run successfully after the latest implementation work:

```bash
cargo test -p carbon-core --features clickhouse clickhouse --lib
cargo check -p carbon-jupiter-swap-decoder --features clickhouse
cargo check -p carbon-token-program-decoder --features clickhouse
cargo check -p jupiter-swap-clickhouse-carbon-example
git diff --check
cargo fmt --all
```

`cargo fmt --all` completed with pre-existing rustfmt warnings about unstable formatting options.
