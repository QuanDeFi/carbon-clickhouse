# Branch Diff: `upstream-v1-sync` To `clickhouse-upstream-v1`

This document is a technical summary of the actual Git diff from `upstream-v1-sync` to the current working tree on `clickhouse-upstream-v1`.

## Diff Basis

Commands used:

```bash
TZ=Asia/Jakarta date '+%Y-%m-%d %H:%M:%S %Z (%z)'
git rev-parse --abbrev-ref HEAD
git rev-parse upstream-v1-sync
git rev-parse HEAD
git merge-base upstream-v1-sync HEAD
git rev-list --left-right --count upstream-v1-sync...HEAD
tmp_index=$(mktemp /tmp/carbon-index.XXXXXX)
cp "$(git rev-parse --git-path index)" "$tmp_index"
GIT_INDEX_FILE="$tmp_index" git add -A
GIT_INDEX_FILE="$tmp_index" git diff --cached --name-status --find-renames upstream-v1-sync
GIT_INDEX_FILE="$tmp_index" git diff --cached --numstat upstream-v1-sync
GIT_INDEX_FILE="$tmp_index" git diff --cached --stat upstream-v1-sync
GIT_INDEX_FILE="$tmp_index" git diff --cached --shortstat upstream-v1-sync
rm -f "$tmp_index"
```

The computation uses a temporary Git index so untracked worktree docs are included without staging the real worktree.

Computed values:

- Diff check timestamp: `2026-05-16 10:17:47 WIB (+0700)`
- Current branch: `clickhouse-upstream-v1`
- Base branch tip: `upstream-v1-sync` at `972028f150ef6b83581a21ddb262c8ed75c27bcf`
- Compare branch tip: `clickhouse-upstream-v1` at `6b84bec8fed69b8cf95acc9fa8cf35d887c2e65e`
- Merge base: `972028f150ef6b83581a21ddb262c8ed75c27bcf`
- Commit relation: `0` commits behind `upstream-v1-sync`, `25` commits ahead
- Changed files: `76`
- Added files: `41`
- Modified files: `35`
- Deleted files: `0`
- Renamed files: `0`
- Insertions: `13112`
- Deletions: `26`

Status legend:

- `[A]` / `A` = added file
- `[M]` / `M` = modified existing file

## Touched Path Tree

```text
[M] .gitignore
[M] Cargo.lock
[M] Cargo.toml
[M] README.md

crates/
└── core/
    ├── [M] Cargo.toml
    └── src/
        ├── [M] account.rs
        ├── [M] account_deletion.rs
        ├── [M] block_details.rs
        ├── [M] instruction.rs
        ├── [M] lib.rs
        ├── [M] pipeline.rs
        ├── [M] processor.rs
        ├── [M] transaction.rs
        └── clickhouse/
            ├── [A] admin.rs
            ├── [A] config.rs
            ├── [A] http.rs
            ├── [A] metrics.rs
            ├── [A] mod.rs
            ├── [A] processors.rs
            ├── [A] writer.rs
            ├── docs/
            │   ├── [A] Branch Diff to v1.md
            │   ├── [A] Carbon Core v1 Architecture.md
            │   ├── [A] Carbon Sink Implementation.md
            │   └── [A] ClickHouse Sink Architecture.md
            └── rows/
                └── [A] mod.rs

datasources/
└── rpc-gpa-datasource/
    └── src/
        └── [M] lib.rs

decoders/
├── jupiter-swap-decoder/
│   ├── [M] Cargo.toml
│   └── src/
│       └── instructions/
│           ├── [M] cpi_event.rs
│           ├── [M] mod.rs
│           └── clickhouse/
│               ├── [A] cpi_event_row.rs
│               ├── [A] instruction_rows.rs
│               └── [A] mod.rs
└── token-program-decoder/
    ├── [M] Cargo.toml
    └── src/
        └── accounts/
            ├── [M] mod.rs
            └── clickhouse/
                ├── [A] mint_row.rs
                ├── [A] mod.rs
                ├── [A] multisig_row.rs
                └── [A] token_row.rs

examples/
├── [M] README.md
├── jupiter-swap-clickhouse/
│   ├── [A] .env.example
│   ├── [A] Cargo.toml
│   ├── [A] README.md
│   └── src/
│       └── [A] main.rs
└── token-program-clickhouse/
    ├── [A] .env.example
    ├── [A] Cargo.toml
    ├── [A] README.md
    └── src/
        └── [A] main.rs

metrics/
└── prometheus-metrics/
    └── src/
        └── [M] lib.rs

monitoring/
├── [A] README.md
├── [A] compose.yaml
├── grafana/
│   ├── dashboards/
│   │   └── [A] carbon-clickhouse-overview.json
│   └── provisioning/
│       ├── dashboards/
│       │   └── [A] dashboards.yml
│       └── datasources/
│           └── [A] prometheus.yml
└── prometheus/
    └── [A] prometheus.yml

packages/
├── cli/
│   └── src/
│       ├── [M] cli.ts
│       └── lib/
│           ├── [M] cargoTomlGenerator.ts
│           ├── [M] decoder.ts
│           ├── [M] prompts.ts
│           └── [M] scaffold.ts
├── renderer/
│   ├── [M] package.json
│   ├── src/
│   │   ├── [M] cargoTomlGenerator.ts
│   │   ├── [A] clickhouseDdl.ts
│   │   ├── [A] clickhouseRowMapper.ts
│   │   ├── [M] getRenderMapVisitor.ts
│   │   └── [M] index.ts
│   ├── templates/
│   │   ├── [A] accountsClickHouseMod.njk
│   │   ├── [M] accountsMod.njk
│   │   ├── [A] clickhouseRowPage.njk
│   │   ├── [A] eventInstructionClickHouseRowPage.njk
│   │   ├── [M] eventInstructionPage.njk
│   │   ├── [A] instructionsClickHouseMod.njk
│   │   ├── [M] instructionsMod.njk
│   │   └── [M] lib.njk
│   └── test/
│       └── [A] clickhouse-renderer.test.cjs
└── versions/
    └── src/
        └── [M] index.ts

scripts/
└── [A] validate-clickhouse-decoder-rollout.sh
```

## Consolidated File-By-File Summary

| Status | +/- | File | Summary |
| --- | ---: | --- | --- |
| M | `+2 / -1` | `.gitignore` | Ignores local ClickHouse architecture visualization output and updates the tracked ignore rules. |
| M | `+48 / -0` | `Cargo.lock` | Locks ClickHouse runtime, renderer, decoder, example, and monitoring-adjacent dependency changes. |
| M | `+1 / -0` | `Cargo.toml` | Adds the workspace `url` dependency used by ClickHouse database URL parsing. |
| M | `+2 / -0` | `README.md` | Documents the ClickHouse decoder rollout validation script and README-driven regeneration path. |
| M | `+5 / -0` | `crates/core/Cargo.toml` | Defines the `clickhouse` feature and optional runtime dependencies. |
| M | `+5 / -1` | `crates/core/src/account.rs` | Adds account pipe finalization and forwards it to account processors. |
| M | `+5 / -1` | `crates/core/src/account_deletion.rs` | Adds account-deletion pipe finalization and forwards it to processors. |
| M | `+5 / -1` | `crates/core/src/block_details.rs` | Adds block-details pipe finalization and forwards it to processors. |
| A | `+55 / -0` | `crates/core/src/clickhouse/admin.rs` | Defines `ClickHouseSchema` and `ClickHouseAdmin` for explicit schema/bootstrap query execution. |
| A | `+379 / -0` | `crates/core/src/clickhouse/config.rs` | Defines ClickHouse connection config, URL parsing, row context construction, sync/async insert settings, batch settings, transport settings, retry settings, and deduplication settings. |
| A | `+261 / -0` | `crates/core/src/clickhouse/docs/Branch Diff to v1.md` | Technical diff document with computed counts, path tree, commit hashes, timestamp, temporary-index computation method, and file summaries. |
| A | `+739 / -0` | `crates/core/src/clickhouse/docs/Carbon Core v1 Architecture.md` | Current-state Carbon core v1 architecture reference for upstream processor surfaces, datasource behavior, pipeline lifecycle, metrics, renderer direction, examples, and future-merge risk. |
| A | `+827 / -0` | `crates/core/src/clickhouse/docs/Carbon Sink Implementation.md` | Current-state implementation reference covering runtime behavior, public runtime APIs, config profiles, generated setup helpers, landing row contracts, writer observability APIs, insert query settings, DDL defaults, validation, production model, monitoring, and rollout policy. |
| A | `+280 / -0` | `crates/core/src/clickhouse/docs/ClickHouse Sink Architecture.md` | ClickHouse sink architecture reference covering design principles, Solana schema boundaries, typed landing-table rationale, identity, batching, responsibility split, rollout boundary, non-goals, and invariants. |
| A | `+233 / -0` | `crates/core/src/clickhouse/http.rs` | Implements authenticated HTTP query/insert helpers, query setting merging, gzip request bodies, query IDs, dedup tokens, and error classification. |
| A | `+285 / -0` | `crates/core/src/clickhouse/metrics.rs` | Registers and records aggregate ClickHouse metrics for instruction and account processor families. |
| A | `+20 / -0` | `crates/core/src/clickhouse/mod.rs` | Re-exports ClickHouse runtime modules, processors, config types, row traits, HTTP helpers, and writer types behind the feature gate. |
| A | `+400 / -0` | `crates/core/src/clickhouse/processors.rs` | Implements ClickHouse instruction and account processors plus processor lifecycle and metrics tests. |
| A | `+109 / -0` | `crates/core/src/clickhouse/rows/mod.rs` | Defines row/table traits, multi-row emission contract, row context, deterministic IDs, and partition helpers for decoder-owned landing rows. |
| A | `+1464 / -0` | `crates/core/src/clickhouse/writer.rs` | Implements per-table/per-partition buffering, serialized byte accounting, backpressure, lazy background flush worker, retry/backoff, failure reinsertion, snapshotting, shutdown drain, and tests. |
| M | `+5 / -0` | `crates/core/src/instruction.rs` | Adds instruction pipe finalization and forwards it to processors. |
| M | `+2 / -0` | `crates/core/src/lib.rs` | Exposes `carbon_core::clickhouse` behind the `clickhouse` feature. |
| M | `+27 / -0` | `crates/core/src/pipeline.rs` | Adds `finalize_pipes()` and calls it on shutdown paths before exporter shutdown. |
| M | `+4 / -0` | `crates/core/src/processor.rs` | Adds default no-op `Processor::finalize()`. |
| M | `+5 / -1` | `crates/core/src/transaction.rs` | Adds transaction pipe finalization and forwards it to processors. |
| M | `+13 / -5` | `datasources/rpc-gpa-datasource/src/lib.rs` | Replaces collapsed GPA error handling with distinct missing-context and RPC error messages. |
| M | `+11 / -0` | `decoders/jupiter-swap-decoder/Cargo.toml` | Adds Jupiter ClickHouse feature/dependencies and test dependencies for generated ClickHouse checks. |
| A | `+606 / -0` | `decoders/jupiter-swap-decoder/src/instructions/clickhouse/cpi_event_row.rs` | Defines typed Jupiter CPI-event landing rows, DDL, structured payload helpers, row conversion, and tests. |
| A | `+1621 / -0` | `decoders/jupiter-swap-decoder/src/instructions/clickhouse/instruction_rows.rs` | Defines typed Jupiter instruction landing rows, DDL, structured route/swap helpers, enum wrappers, serializers, conversions, and tests. |
| A | `+536 / -0` | `decoders/jupiter-swap-decoder/src/instructions/clickhouse/mod.rs` | Wires Jupiter instruction/event row enum dispatch, migrations, setup helpers, config helpers, and processor alias. |
| M | `+19 / -0` | `decoders/jupiter-swap-decoder/src/instructions/cpi_event.rs` | Adds CPI-event account construction from raw instruction accounts. |
| M | `+14 / -1` | `decoders/jupiter-swap-decoder/src/instructions/mod.rs` | Exposes the ClickHouse module and decodes CPI events through an explicit pre-check path. |
| M | `+7 / -0` | `decoders/token-program-decoder/Cargo.toml` | Adds Token Program ClickHouse feature and optional dependencies. |
| A | `+156 / -0` | `decoders/token-program-decoder/src/accounts/clickhouse/mint_row.rs` | Defines typed Token Program mint account ClickHouse row, DDL, conversion, and partitioning. |
| A | `+172 / -0` | `decoders/token-program-decoder/src/accounts/clickhouse/mod.rs` | Wires Token Program account row enum dispatch, migrations, setup helpers, config helpers, and processor alias. |
| A | `+156 / -0` | `decoders/token-program-decoder/src/accounts/clickhouse/multisig_row.rs` | Defines typed Token Program multisig account ClickHouse row, DDL, conversion, and partitioning. |
| A | `+195 / -0` | `decoders/token-program-decoder/src/accounts/clickhouse/token_row.rs` | Defines typed Token Program token account ClickHouse row, DDL, conversion, enum formatting helpers, and partitioning. |
| M | `+3 / -0` | `decoders/token-program-decoder/src/accounts/mod.rs` | Exposes generated account ClickHouse module behind the feature gate. |
| M | `+1 / -0` | `examples/README.md` | Lists the Token Program ClickHouse example in the examples index. |
| A | `+20 / -0` | `examples/jupiter-swap-clickhouse/.env.example` | Documents environment variables for the Jupiter ClickHouse example, including provider RPC guidance. |
| A | `+19 / -0` | `examples/jupiter-swap-clickhouse/Cargo.toml` | Defines the Jupiter ClickHouse example crate and dependencies. |
| A | `+36 / -0` | `examples/jupiter-swap-clickhouse/README.md` | Documents the Jupiter ClickHouse smoke example, provider RPC environment, run command, and validation queries. |
| A | `+113 / -0` | `examples/jupiter-swap-clickhouse/src/main.rs` | Runs bounded RPC block crawling into generated Jupiter instruction and CPI-event rows. |
| A | `+31 / -0` | `examples/token-program-clickhouse/.env.example` | Documents environment variables for the Token Program ClickHouse example, including provider RPC guidance. |
| A | `+23 / -0` | `examples/token-program-clickhouse/Cargo.toml` | Defines the Token Program ClickHouse example crate and dependencies. |
| A | `+68 / -0` | `examples/token-program-clickhouse/README.md` | Documents the Token Program ClickHouse account example, provider RPC environment, filters, run modes, and async insert option. |
| A | `+255 / -0` | `examples/token-program-clickhouse/src/main.rs` | Runs filtered Token Program account snapshots through RPC or Helius GPA into generated ClickHouse account rows. |
| M | `+22 / -1` | `metrics/prometheus-metrics/src/lib.rs` | Adjusts Prometheus metric export handling for Carbon metric names used by the ClickHouse monitoring stack. |
| A | `+101 / -0` | `monitoring/README.md` | Documents local Prometheus/Grafana setup, access, scraping behavior, and common queries. |
| A | `+37 / -0` | `monitoring/compose.yaml` | Defines the local Prometheus and Grafana Docker Compose stack for Carbon metrics. |
| A | `+490 / -0` | `monitoring/grafana/dashboards/carbon-clickhouse-overview.json` | Adds a Grafana dashboard for Carbon process and ClickHouse sink metrics. |
| A | `+11 / -0` | `monitoring/grafana/provisioning/dashboards/dashboards.yml` | Provisions the Carbon Grafana dashboard folder. |
| A | `+10 / -0` | `monitoring/grafana/provisioning/datasources/prometheus.yml` | Provisions Prometheus as the Grafana datasource. |
| A | `+15 / -0` | `monitoring/prometheus/prometheus.yml` | Configures Prometheus to scrape local Carbon metrics. |
| M | `+9 / -2` | `packages/cli/src/cli.ts` | Threads ClickHouse renderer options through CLI command parsing and generation. |
| M | `+10 / -1` | `packages/cli/src/lib/cargoTomlGenerator.ts` | Adds CLI-side manifest support for ClickHouse generation and dependencies. |
| M | `+4 / -0` | `packages/cli/src/lib/decoder.ts` | Extends decoder generation options with ClickHouse support. |
| M | `+2 / -0` | `packages/cli/src/lib/prompts.ts` | Adds ClickHouse prompt/config handling. |
| M | `+3 / -0` | `packages/cli/src/lib/scaffold.ts` | Propagates ClickHouse scaffold options into generated decoder projects. |
| M | `+1 / -1` | `packages/renderer/package.json` | Replaces the placeholder renderer test script with build plus ClickHouse renderer test execution. |
| M | `+19 / -2` | `packages/renderer/src/cargoTomlGenerator.ts` | Adds `withClickHouse` manifest option, ClickHouse feature generation, and optional `chrono` dependency generation. |
| A | `+196 / -0` | `packages/renderer/src/clickhouseDdl.ts` | Implements renderer-controlled ClickHouse DDL planning for MergeTree, ReplicatedMergeTree, Distributed tables, clusters, keys, TTLs, engine settings, codecs, and additive migrations. |
| A | `+1073 / -0` | `packages/renderer/src/clickhouseRowMapper.ts` | Implements schema planning for ClickHouse row fields, DDL types, Rust helper types, imports, conversion expressions, strict fallback behavior, and structured enum/composite handling. |
| M | `+87 / -1` | `packages/renderer/src/getRenderMapVisitor.ts` | Hooks `withClickHouse` into account, instruction, CPI-event row rendering, module generation, DDL options, and JSON fallback behavior. |
| M | `+4 / -0` | `packages/renderer/src/index.ts` | Exports ClickHouse row mapper and DDL planning types. |
| A | `+152 / -0` | `packages/renderer/templates/accountsClickHouseMod.njk` | Generates account ClickHouse module dispatch, migrations, setup helpers, and processor wiring. |
| M | `+6 / -1` | `packages/renderer/templates/accountsMod.njk` | Emits account ClickHouse module behind `#[cfg(feature = "clickhouse")]`. |
| A | `+384 / -0` | `packages/renderer/templates/clickhouseRowPage.njk` | Generates typed account/instruction ClickHouse row structs, DDL, migrations, conversions, helper types, and row trait impls. |
| A | `+271 / -0` | `packages/renderer/templates/eventInstructionClickHouseRowPage.njk` | Generates typed CPI-event ClickHouse row structs, DDL, migrations, conversions, helper types, and row trait impls. |
| M | `+17 / -1` | `packages/renderer/templates/eventInstructionPage.njk` | Generates CPI-event account arrangement helpers for event instructions. |
| A | `+206 / -0` | `packages/renderer/templates/instructionsClickHouseMod.njk` | Generates instruction/CPI-event ClickHouse row enum dispatch, migrations, setup helpers, and processor alias. |
| M | `+20 / -4` | `packages/renderer/templates/instructionsMod.njk` | Emits ClickHouse module support and CPI-event pre-check decoding. |
| M | `+1 / -1` | `packages/renderer/templates/lib.njk` | Keeps generated decoder root exports aligned with account, instruction, event, type, GraphQL, and ClickHouse modules. |
| A | `+365 / -0` | `packages/renderer/test/clickhouse-renderer.test.cjs` | Adds renderer coverage for ClickHouse rows, DDL modes, structured mapper behavior, strict fallback behavior, and generated account/instruction modules. |
| M | `+1 / -0` | `packages/versions/src/index.ts` | Updates package version metadata for the ClickHouse-enabled CLI/rendering path. |
| A | `+352 / -0` | `scripts/validate-clickhouse-decoder-rollout.sh` | Adds a non-committing ClickHouse decoder rollout validation script for canary checks, broad compile scans, and temporary IDL/README regeneration checks. |
