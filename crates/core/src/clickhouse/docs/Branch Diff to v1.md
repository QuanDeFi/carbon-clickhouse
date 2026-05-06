# Branch Diff: `upstream-v1-sync` To `clickhouse-upstream-v1`

This document is a technical summary of the actual Git diff from `upstream-v1-sync` to the working tree on `clickhouse-upstream-v1`.

## Diff Basis

Commands used:

```bash
git rev-parse --abbrev-ref HEAD
git merge-base upstream-v1-sync HEAD
git rev-list --left-right --count upstream-v1-sync...HEAD
git diff --name-status --find-renames upstream-v1-sync
git diff --numstat upstream-v1-sync
git diff --stat upstream-v1-sync
git diff --shortstat upstream-v1-sync
```

All reported files are tracked in Git; no intent-to-add step is required for the current computation.

Computed values:

- Current branch: `clickhouse-upstream-v1`
- Merge base: `972028f150ef6b83581a21ddb262c8ed75c27bcf`
- Commit relation: `0` commits behind `upstream-v1-sync`, `18` commits ahead
- Changed files: `57`
- Added files: `30`
- Modified files: `27`
- Deleted files: `0`
- Renamed files: `0`
- Insertions: `9265`
- Deletions: `22`

## Touched Path Tree

```text
[M] .gitignore
[M] Cargo.lock
[M] Cargo.toml

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
            │   ├── [A] Carbon Sink Implementation.md
            │   └── [A] architecture.md
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
│   ├── [A] Cargo.toml
│   └── src/
│       └── [A] main.rs
└── token-program-clickhouse/
    ├── [A] .env.example
    ├── [A] Cargo.toml
    ├── [A] README.md
    └── src/
        └── [A] main.rs

packages/
└── renderer/
    ├── [M] package.json
    ├── src/
    │   ├── [M] cargoTomlGenerator.ts
    │   ├── [A] clickhouseRowMapper.ts
    │   ├── [M] getRenderMapVisitor.ts
    │   └── [M] index.ts
    ├── templates/
    │   ├── [A] accountsClickHouseMod.njk
    │   ├── [M] accountsMod.njk
    │   ├── [A] clickhouseRowPage.njk
    │   ├── [A] eventInstructionClickHouseRowPage.njk
    │   ├── [M] eventInstructionPage.njk
    │   ├── [A] instructionsClickHouseMod.njk
    │   ├── [M] instructionsMod.njk
    │   └── [M] lib.njk
    └── test/
        └── [A] clickhouse-renderer.test.cjs
```

## Consolidated File-By-File Summary

| Status | +/- | File | Summary |
| --- | ---: | --- | --- |
| M | `+2 / -1` | `.gitignore` | Ignores the local ClickHouse architecture visualization document and normalizes the final newline. |
| M | `+45 / -0` | `Cargo.lock` | Locks the ClickHouse core dependencies, decoder feature dependencies, and ClickHouse example crates. |
| M | `+1 / -0` | `Cargo.toml` | Adds workspace `url` dependency used by ClickHouse database URL parsing. |
| M | `+4 / -0` | `crates/core/Cargo.toml` | Defines the `clickhouse` feature and optional `reqwest`, `sha2`, and `url` dependencies. |
| M | `+5 / -1` | `crates/core/src/account.rs` | Adds account pipe finalization and forwards it to account processors. |
| M | `+5 / -1` | `crates/core/src/account_deletion.rs` | Adds account-deletion pipe finalization and forwards it to processors. |
| M | `+5 / -1` | `crates/core/src/block_details.rs` | Adds block-details pipe finalization and forwards it to processors. |
| A | `+50 / -0` | `crates/core/src/clickhouse/admin.rs` | Defines `ClickHouseSchema` and `ClickHouseAdmin` for explicit schema/bootstrap query execution. |
| A | `+245 / -0` | `crates/core/src/clickhouse/config.rs` | Defines ClickHouse connection config, URL parsing, row context construction, sync insert defaults, and async-wait insert settings. |
| A | `+189 / -0` | `crates/core/src/clickhouse/docs/Branch Diff to v1.md` | Technical diff document with computed counts, path tree, and file summaries. |
| A | `+276 / -0` | `crates/core/src/clickhouse/docs/Carbon Sink Implementation.md` | Current-state ClickHouse sink implementation reference. |
| A | `+525 / -0` | `crates/core/src/clickhouse/docs/architecture.md` | ClickHouse sink architecture reference covering layers, lifecycle, table design, identity, metrics, and boundaries. |
| A | `+78 / -0` | `crates/core/src/clickhouse/http.rs` | Implements authenticated HTTP query and insert helpers with shared ClickHouse query settings. |
| A | `+163 / -0` | `crates/core/src/clickhouse/metrics.rs` | Registers and records ClickHouse metrics for instruction and account families. |
| A | `+13 / -0` | `crates/core/src/clickhouse/mod.rs` | Re-exports the ClickHouse runtime modules, processors, and writer behind the feature gate. |
| A | `+400 / -0` | `crates/core/src/clickhouse/processors.rs` | Implements ClickHouse processors for instruction and account rows plus processor tests. |
| A | `+109 / -0` | `crates/core/src/clickhouse/rows/mod.rs` | Defines row/table traits, multi-row emission contract, row context, and deterministic ID helpers for decoder-owned landing rows. |
| A | `+841 / -0` | `crates/core/src/clickhouse/writer.rs` | Implements the per-table/per-partition buffered writer, lazy background flush worker, failure reinsertion, shutdown drain, insert batching, and tests. |
| M | `+5 / -0` | `crates/core/src/instruction.rs` | Adds instruction pipe finalization and forwards it to processors. |
| M | `+2 / -0` | `crates/core/src/lib.rs` | Exposes `carbon_core::clickhouse` behind the `clickhouse` feature. |
| M | `+27 / -0` | `crates/core/src/pipeline.rs` | Adds `finalize_pipes()` and calls it on shutdown paths before exporter shutdown. |
| M | `+4 / -0` | `crates/core/src/processor.rs` | Adds default no-op `Processor::finalize()`. |
| M | `+5 / -1` | `crates/core/src/transaction.rs` | Adds transaction pipe finalization and forwards it to processors. |
| M | `+13 / -5` | `datasources/rpc-gpa-datasource/src/lib.rs` | Replaces collapsed GPA error handling with distinct missing-context and RPC error messages. |
| M | `+11 / -0` | `decoders/jupiter-swap-decoder/Cargo.toml` | Adds Jupiter ClickHouse feature/dependencies and test support dependencies for generated ClickHouse checks. |
| A | `+606 / -0` | `decoders/jupiter-swap-decoder/src/instructions/clickhouse/cpi_event_row.rs` | Defines typed Jupiter CPI-event landing rows, DDL, structured payload helpers, row conversion, and tests. |
| A | `+1621 / -0` | `decoders/jupiter-swap-decoder/src/instructions/clickhouse/instruction_rows.rs` | Defines typed Jupiter instruction landing rows, DDL, structured route/swap helper types, enum wrappers, serializers, conversions, and tests. |
| A | `+536 / -0` | `decoders/jupiter-swap-decoder/src/instructions/clickhouse/mod.rs` | Wires Jupiter instruction/event row enum dispatch, migrations, setup helpers, config helpers, and processor alias. |
| M | `+19 / -0` | `decoders/jupiter-swap-decoder/src/instructions/cpi_event.rs` | Adds CPI-event account construction from raw instruction accounts. |
| M | `+14 / -1` | `decoders/jupiter-swap-decoder/src/instructions/mod.rs` | Exposes the ClickHouse module and decodes CPI events through an explicit pre-check path. |
| M | `+7 / -0` | `decoders/token-program-decoder/Cargo.toml` | Adds Token Program ClickHouse feature and optional dependencies. |
| A | `+156 / -0` | `decoders/token-program-decoder/src/accounts/clickhouse/mint_row.rs` | Defines typed Token Program mint account ClickHouse row, DDL, conversion, and partitioning. |
| A | `+172 / -0` | `decoders/token-program-decoder/src/accounts/clickhouse/mod.rs` | Wires Token Program account row enum dispatch, migrations, setup helpers, config helpers, and processor alias. |
| A | `+156 / -0` | `decoders/token-program-decoder/src/accounts/clickhouse/multisig_row.rs` | Defines typed Token Program multisig account ClickHouse row, DDL, conversion, and partitioning. |
| A | `+195 / -0` | `decoders/token-program-decoder/src/accounts/clickhouse/token_row.rs` | Defines typed Token Program token account ClickHouse row, DDL, conversion, enum formatting helpers, and partitioning. |
| M | `+3 / -0` | `decoders/token-program-decoder/src/accounts/mod.rs` | Exposes generated account ClickHouse module behind the feature flag. |
| M | `+1 / -0` | `examples/README.md` | Lists the Token Program ClickHouse example in the examples index. |
| A | `+18 / -0` | `examples/jupiter-swap-clickhouse/Cargo.toml` | Defines the Jupiter ClickHouse example crate and dependencies. |
| A | `+100 / -0` | `examples/jupiter-swap-clickhouse/src/main.rs` | Runs bounded RPC block crawling into generated Jupiter instruction and CPI-event rows. |
| A | `+26 / -0` | `examples/token-program-clickhouse/.env.example` | Documents environment variables for the Token Program ClickHouse example. |
| A | `+22 / -0` | `examples/token-program-clickhouse/Cargo.toml` | Defines the Token Program ClickHouse example crate and dependencies. |
| A | `+60 / -0` | `examples/token-program-clickhouse/README.md` | Documents the Token Program ClickHouse account example, filters, run modes, and async insert option. |
| A | `+242 / -0` | `examples/token-program-clickhouse/src/main.rs` | Runs filtered Token Program account snapshots through RPC or Helius GPA into generated ClickHouse account rows. |
| M | `+1 / -1` | `packages/renderer/package.json` | Replaces placeholder renderer test script with build plus ClickHouse renderer test execution. |
| M | `+19 / -2` | `packages/renderer/src/cargoTomlGenerator.ts` | Adds `withClickHouse` manifest option, ClickHouse feature generation, and optional `chrono` dependency generation. |
| A | `+1061 / -0` | `packages/renderer/src/clickhouseRowMapper.ts` | Implements schema planning for ClickHouse row fields, DDL types, Rust helper types, imports, and conversion expressions. |
| M | `+74 / -1` | `packages/renderer/src/getRenderMapVisitor.ts` | Hooks `withClickHouse` into account, instruction, and CPI-event row rendering and module generation. |
| M | `+2 / -0` | `packages/renderer/src/index.ts` | Exports `ClickHouseRowMapper` and related types. |
| A | `+152 / -0` | `packages/renderer/templates/accountsClickHouseMod.njk` | Generates account ClickHouse module dispatch, migrations, setup helpers, and processor wiring. |
| M | `+6 / -1` | `packages/renderer/templates/accountsMod.njk` | Emits account ClickHouse module behind `#[cfg(feature = "clickhouse")]`. |
| A | `+298 / -0` | `packages/renderer/templates/clickhouseRowPage.njk` | Generates typed account/instruction ClickHouse row structs, DDL, conversions, and row trait impls. |
| A | `+195 / -0` | `packages/renderer/templates/eventInstructionClickHouseRowPage.njk` | Generates typed CPI-event ClickHouse row structs, DDL, conversions, and row trait impls. |
| M | `+17 / -1` | `packages/renderer/templates/eventInstructionPage.njk` | Generates CPI-event account arrangement helpers for event instructions. |
| A | `+206 / -0` | `packages/renderer/templates/instructionsClickHouseMod.njk` | Generates instruction/CPI-event ClickHouse row enum dispatch, migrations, setup helpers, and processor alias. |
| M | `+20 / -4` | `packages/renderer/templates/instructionsMod.njk` | Emits ClickHouse module support and CPI-event pre-check decoding. |
| M | `+1 / -1` | `packages/renderer/templates/lib.njk` | Keeps generated decoder root exports aligned with account, instruction, event, type, and GraphQL modules. |
| A | `+236 / -0` | `packages/renderer/test/clickhouse-renderer.test.cjs` | Adds renderer coverage for ClickHouse rows, structured mapper behavior, and generated account/instruction modules. |
