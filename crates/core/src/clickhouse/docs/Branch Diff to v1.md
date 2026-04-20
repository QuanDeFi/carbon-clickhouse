## Branch diff upstream-v1-sync → clickhouse-upstream-v1


## Overall shape

* Branch relation:

  * `clickhouse-upstream-v1` is **ahead by 4 commits**
  * `behind by 0`
  * merge base is exactly `upstream-v1-sync`
* File counts:

  * Added files: **13**
  * Modified files: **20**
  * Removed files: **0**

Core meaning of the branch:

* it adds a **new ClickHouse backend path** in `carbon-core`
* it adds **processor finalization + pipeline shutdown flushing**
* it adds a **Jupiter swap CPI-event → ClickHouse landing-table** implementation
* it adds a **working example**
* it adds **renderer/codegen support** so this is not just a one-off hand patch

## Touched path tree

```text
[M] Cargo.lock                        — lockfile / pinned dependency state
[M] Cargo.toml                        — workspace manifest

crates/
├── core/
│   ├── [M] Cargo.toml                — core crate features + optional deps
│   └── src/
│       ├── [M] account.rs            — account pipe wrapper
│       ├── [M] account_deletion.rs   — account-deletion pipe wrapper
│       ├── [M] block_details.rs      — block-level pipe trait + wrapper
│       ├── [M] instruction.rs        — instruction metadata, log decoding, nested CPI tree, instruction pipe trait
│       ├── [M] lib.rs                — carbon-core public module exports
│       ├── [M] pipeline.rs           — pipeline runtime + shutdown orchestration
│       ├── [M] processor.rs          — generic processor trait
│       ├── [M] transaction.rs        — transaction metadata + transaction pipe wrapper
│       └── clickhouse/
│           ├── [A] admin.rs          — schema/admin query executor
│           ├── [A] config.rs         — ClickHouse sink configuration
│           ├── [A] http.rs           — low-level HTTP transport helpers
│           ├── [A] mod.rs            — ClickHouse module barrel + public re-exports
│           ├── [A] processors.rs     — instruction-to-ClickHouse sink processor
│           ├── [A] writer.rs         — buffered batch writer for ClickHouse inserts
│           ├── docs/
│           │   └── [A] architecture.md — ClickHouse sink design document
│           └── rows/
│               └── [A] mod.rs        — row/table traits + deterministic event IDs
└── proc-macros/
    └── src/
        └── [M] lib.rs                — decoder-collection proc macros

decoders/
└── jupiter-swap-decoder/
    ├── [M] Cargo.toml                — decoder crate manifest
    └── src/
        └── instructions/
            ├── [M] cpi_event.rs      — CPI-event type + account arrangement
            ├── [M] mod.rs            — Jupiter instruction entrypoint/decoder
            └── clickhouse/
                ├── [A] cpi_event_row.rs — typed ClickHouse landing row
                └── [A] mod.rs        — Jupiter ClickHouse glue + bootstrap

examples/
└── jupiter-swap-clickhouse/
    ├── [A] Cargo.toml                — example crate manifest
    └── src/
        └── [A] main.rs               — end-to-end ClickHouse example

packages/
└── renderer/
    ├── src/
    │   ├── [M] cargoTomlGenerator.ts — gen Cargo.toml feature/dependency builder
    │   └── [M] getRenderMapVisitor.ts — main decoder/codegen render orchestrator
    └── templates/
        ├── [A] eventInstructionClickHouseRowPage.njk — gen ClickHouse CPI-event row templ
        ├── [M] eventInstructionPage.njk — gen CPI-event decoder templ
        ├── [A] instructionsClickHouseMod.njk — gen ClickHouse instruction glue templ
        └── [M] instructionsMod.njk — gen instruction module/decoder templ
```

Legend:

[A] added
[M] modified

## Consolidated file-by-file summary

### Workspace and top-level files

| File                     | Patch size | What it is                                 | Diff vs `upstream-v1-sync`                                                                                                                                                                      |
| ------------------------ | ---------: | ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Cargo.lock`             | `+24 / -0` | Generated Rust lockfile for the workspace. | Generated dependency update only. It pins the dependency graph implied by the new ClickHouse-related manifest changes. No meaningful hand-written logic.                                        |
| `Cargo.toml`             |  `+1 / -0` | Top-level workspace manifest.              | Small diff: adds `reqwest` at workspace level so the new core ClickHouse backend can use HTTP transport.                                                                                        |
| `crates/core/Cargo.toml` |  `+4 / -0` | Manifest for `carbon-core`.                | Adds a new `clickhouse` feature and wires it to optional deps `reqwest`, `sha2`, and `url`, making ClickHouse a first-class optional backend alongside existing ones like Postgres and GraphQL. |

### Core pipe wrappers and exports

| File                                  | Patch size | What it is                                                                                                                                                                    | Diff vs `upstream-v1-sync`                                                                                                                                                                                                   |
| ------------------------------------- | ---------: | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/core/src/account.rs`          |  `+5 / -0` | Account decoding/processing abstractions: decoder, pipe wrapper, pipe trait, processor input wrapper.                                                                         | Lifecycle-only change: adds `finalize()` to `AccountPipes` and forwards it to the underlying processor. No account decoding semantics changed.                                                                               |
| `crates/core/src/account_deletion.rs` |  `+5 / -0` | Account-deletion processing wrapper and trait.                                                                                                                                | Same pattern: adds `finalize()` to `AccountDeletionPipes` and forwards it. No deletion logic changed.                                                                                                                        |
| `crates/core/src/block_details.rs`    |  `+5 / -0` | Block-details pipe wrapper and trait for block-level processors.                                                                                                              | Exact diff is just lifecycle propagation: trait gains `finalize()`, impl calls `self.processor.finalize().await`. `run()` is unchanged.                                                                                      |
| `crates/core/src/instruction.rs`      |  `+5 / -0` | Main instruction-processing module: instruction metadata, log event extraction, precompile handling, nested CPI reconstruction, decoder traits, processor input types, tests. | Only the instruction pipe lifecycle changed: `InstructionPipes<'a>` gains `finalize()`, and `InstructionPipe<T, P>` forwards it. Log parsing, nested instruction logic, event extraction, and tests are otherwise unchanged. |
| `crates/core/src/lib.rs`              |  `+4 / -2` | Public export surface of `carbon-core`.                                                                                                                                       | Adds `#[cfg(feature = "clickhouse")] pub mod clickhouse;`, exposing the new backend module. Existing `postgres`/`graphql` exports remain. This is a tiny public-surface change.                                              |

### Core runtime and lifecycle

| File                             | Patch size | What it is                                                                                                                                         | Diff vs `upstream-v1-sync`                                                                                                                                                                                                                                                                                                         |
| -------------------------------- | ---------: | -------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/core/src/pipeline.rs`    | `+28 / -1` | Main Carbon runtime: owns datasources and pipes, runs the event loop, applies filters, builds nested instructions, handles shutdown and exporters. | Biggest core runtime change. Adds `finalize_pipes(&mut self)` and calls it on all shutdown paths: datasource cancellation, Ctrl-C with `Immediate`, and receiver close. In upstream, shutdown only exported metrics and shut down exporters. This is the key change that makes buffered sinks flush deterministically before exit. |
| `crates/core/src/processor.rs`   |  `+4 / -0` | Base trait every Carbon processor implements.                                                                                                      | Adds default `finalize()` returning `Ok(())`. Existing processors stay compatible; buffered processors can override it. This is the root lifecycle hook that enables the shutdown change.                                                                                                                                          |
| `crates/core/src/transaction.rs` |  `+5 / -0` | Transaction metadata, conversion from datasource updates, flat instruction parsing, transaction pipe wrapper/trait.                                | Same lifecycle pattern as other pipe wrappers: trait gains `finalize()`, impl forwards it. Transaction parsing and metadata logic are otherwise unchanged.                                                                                                                                                                         |

### New core ClickHouse backend

| File                                              |  Patch size | What it is                                                                                                                           | Diff vs `upstream-v1-sync`                                                                                                                                                                                                                                                                           |
| ------------------------------------------------- | ----------: | ------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/core/src/clickhouse/admin.rs`             |  `+47 / -0` | Schema/bootstrap executor. Defines `ClickHouseSchema` and `ClickHouseAdmin`, which can execute one query, many queries, or a schema. | New file. Introduces a dedicated ClickHouse admin/bootstrap abstraction. Uses `post_query(...)` and always passes `date_time_input_format=best_effort`.                                                                                                                                              |
| `crates/core/src/clickhouse/config.rs`            | `+102 / -0` | Central sink config object. Stores endpoint, DB, auth, table, source/mode/version metadata, and batching controls.                   | New file. Adds `from_database_url(...)` for parsing a DB URL into config and `row_context()` for exposing just the row-generation metadata.                                                                                                                                                          |
| `crates/core/src/clickhouse/http.rs`              |  `+78 / -0` | Low-level HTTP transport helpers.                                                                                                    | New file. Adds `apply_auth(...)`, `post_query(...)`, and `post_query_with_data(...)`. Splits “execute SQL” from “execute SQL + data payload” and centralizes error handling.                                                                                                                         |
| `crates/core/src/clickhouse/mod.rs`               |  `+11 / -0` | ClickHouse module barrel and public re-exports.                                                                                      | New file. Exposes `admin`, `config`, `http`, `processors`, `rows`, `writer`, plus the main public API.                                                                                                                                                                                               |
| `crates/core/src/clickhouse/processors.rs`        | `+175 / -0` | Core bridge from Carbon instruction processing into ClickHouse writes.                                                               | New file. Defines ClickHouse metrics, registers them once, implements `ClickHouseInstructionProcessor<T, W, R>`, converts decoded instructions into row wrappers, buffers rows, records flush metrics, and overrides `finalize()` to flush on shutdown. This is one of the key new behavioral files. |
| `crates/core/src/clickhouse/writer.rs`            | `+175 / -0` | Buffered batch writer for ClickHouse inserts.                                                                                        | New file. Implements `ClickHouseBatchWriter<R>` with in-memory buffers grouped by partition key, size/time-based flushing, JSONEachRow serialization, and batch reinsertion on failed flush. This is the actual batching engine behind the sink.                                                     |
| `crates/core/src/clickhouse/docs/architecture.md` | `+480 / -0` | Design document for the ClickHouse sink.                                                                                             | New file. Explains the architecture and intended scope: processor-driven sink, client-side batching, landing-only tables, deterministic event IDs, decoder-owned schema, and finalize-on-shutdown as the main core change.                                                                           |
| `crates/core/src/clickhouse/rows/mod.rs`          |  `+63 / -0` | Row/table contracts and deterministic event ID support.                                                                              | New file. Defines `ClickHouseTable`, `ClickHouseRow`, `ClickHouseRowContext`, `ClickHouseRows<R>`, and `deterministic_event_id(...)` using SHA-256 over program/signature/path/type/seq.                                                                                                             |

### Proc macros

| File                            | Patch size | What it is                                                                  | Diff vs `upstream-v1-sync`                                                                                                                                                                                                                                                                                                        |
| ------------------------------- | ---------: | --------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/proc-macros/src/lib.rs` |  `+1 / -2` | Proc macros for instruction decoder collections and instruction-type enums. | Tiny modified file. Git shows a real diff, so it is **not literally identical**. But from the fetched content, there is **no obvious semantic change**: same 3-part/4-part syntax support, same dispatch generation, same derive macro. Best wording is: **tiny textual diff, no clear behavioral delta from inspected content**. |

### Jupiter decoder and ClickHouse-specific decoder wiring

| File                                                                         |  Patch size | What it is                                                                                                                        | Diff vs `upstream-v1-sync`                                                                                                                                                                                                                                                                                                                                        |
| ---------------------------------------------------------------------------- | ----------: | --------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `decoders/jupiter-swap-decoder/Cargo.toml`                                   |  `+11 / -0` | Decoder crate manifest.                                                                                                           | Adds decoder-side ClickHouse feature/dependencies.                                                                                                                                                                                                                                                                                                                |
| `decoders/jupiter-swap-decoder/src/instructions/cpi_event.rs`                |  `+19 / -0` | Autogenerated Jupiter CPI-event decoder. Defines `CpiEvent`, `CpiEventInstructionAccounts`, `decode(...)`, and `ArrangeAccounts`. | Adds `CpiEventInstructionAccounts::from_instruction_accounts(program_id, accounts)`, which allows direct construction outside the `ArrangeAccounts` path. The actual event decode logic is otherwise the same.                                                                                                                                                    |
| `decoders/jupiter-swap-decoder/src/instructions/mod.rs`                      |  `+14 / -1` | Main Jupiter instruction entrypoint/decoder. Defines instruction enum and `InstructionDecoder` impl.                              | Two real changes. First, adds `#[cfg(feature = "clickhouse")] pub mod clickhouse;`. Second, moves CPI-event handling to an explicit pre-check: it calls `CpiEvent::decode(...)` before `try_decode_instructions!`, and builds `JupiterSwapInstruction::CpiEvent` via `from_instruction_accounts(...)`. In upstream, `CpiEvent` was decoded through the macro arm. |
| `decoders/jupiter-swap-decoder/src/instructions/clickhouse/cpi_event_row.rs` | `+280 / -0` | Jupiter-specific typed ClickHouse landing row.                                                                                    | New file. Defines `JupiterSwapSwapEventLandingRow`, maps `InstructionMetadata + SwapEventEvent + ClickHouseRowContext` into a row, computes deterministic `event_id`, chooses `partition_time`, defines MergeTree DDL, and includes stability tests.                                                                                                              |
| `decoders/jupiter-swap-decoder/src/instructions/clickhouse/mod.rs`           | `+128 / -0` | Jupiter-specific ClickHouse glue and bootstrap layer.                                                                             | New file. Exports the row type, defines defaults, defines migration/bootstrap helpers, wraps decoded instruction + metadata + accounts, implements `ClickHouseRows<JupiterSwapSwapEventLandingRow>`, and exposes setup helpers. This is the decoder-owned adapter from generic core ClickHouse support to one concrete Jupiter family.                            |

### Example

| File                                           | Patch size | What it is              | Diff vs `upstream-v1-sync`                                                                                                                                                                                                                                                                                                              |
| ---------------------------------------------- | ---------: | ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `examples/jupiter-swap-clickhouse/Cargo.toml`  | `+18 / -0` | Example crate manifest. | New example manifest.                                                                                                                                                                                                                                                                                                                   |
| `examples/jupiter-swap-clickhouse/src/main.rs` | `+97 / -0` | End-to-end example.     | New file. Loads env/config, requires `DATABASE_URL`, sets up ClickHouse, creates an `RpcBlockCrawler`, attaches `JupiterSwapDecoder` with the ClickHouse processor, adds log metrics, and runs the pipeline with `ShutdownStrategy::Immediate`. It proves the bounded backfill path from RPC blocks to ClickHouse landing-table writes. |

### Renderer / generator

| File                                                                |  Patch size | What it is                                                           | Diff vs `upstream-v1-sync`                                                                                                                                                                                                                                                               |
| ------------------------------------------------------------------- | ----------: | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `packages/renderer/src/cargoTomlGenerator.ts`                       |  `+19 / -2` | Generates decoder `Cargo.toml` files.                                | Adds `withClickHouse?: boolean`, emits a `clickhouse` feature, includes `carbon-core/clickhouse`, `serde`, and `chrono`, and adds `chrono` dependency when ClickHouse generation is enabled. This is the manifest-generation part of ClickHouse support.                                 |
| `packages/renderer/src/getRenderMapVisitor.ts`                      |  `+17 / -0` | Main generator orchestrator for decoder code.                        | Adds `withClickHouse?: boolean` and conditional generation of `src/instructions/clickhouse/mod.rs` and `src/instructions/clickhouse/cpi_event_row.rs`. Also passes `withClickHouse` into Cargo generation. This turns ClickHouse into a real generator target, not a one-off hand patch. |
| `packages/renderer/templates/eventInstructionClickHouseRowPage.njk` | `+142 / -0` | Template for generating a generic ClickHouse CPI-event landing row.  | New file. Generates a `CpiEventRow` with common chain/sink metadata, deterministic identity, JSON event payload, MergeTree DDL, and year-based partitioning.                                                                                                                             |
| `packages/renderer/templates/eventInstructionPage.njk`              |  `+17 / -1` | Template for generating CPI-event decoder modules.                   | Modified to add `CpiEventInstructionAccounts::from_instruction_accounts(program_id, accounts)`. The rest of the CPI-event decoding structure remains the same.                                                                                                                           |
| `packages/renderer/templates/instructionsClickHouseMod.njk`         |  `+54 / -0` | Template for generating decoder-side ClickHouse instruction glue.    | New file. Generates wrapper type, migration type, and `ClickHouseRows<CpiEventRow>` impl for decoder instruction modules.                                                                                                                                                                |
| `packages/renderer/templates/instructionsMod.njk`                   |  `+20 / -4` | Template for generating `src/instructions/mod.rs` in decoder crates. | Two important changes: it conditionally emits `#[cfg(feature = "clickhouse")] pub mod clickhouse;`, and it changes CPI-event decoding to an explicit pre-check before the generic instruction macro. In upstream, CPI events were handled inside the macro dispatch.                     |

## What the direct compare clarifies

| Area                | Updated interpretation                                                                                                                                                                                                             |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Core pipe wrappers  | These are confirmed to be very small lifecycle propagations: mostly **`+5` lines each** in `account.rs`, `account_deletion.rs`, `block_details.rs`, `instruction.rs`, and `transaction.rs`.                                        |
| `pipeline.rs`       | Still the most important core runtime modification, and the compare confirms it is compact but real: **`+28 / -1`**.                                                                                                               |
| `processor.rs`      | The lifecycle hook itself is very small and surgical: **`+4 / -0`**.                                                                                                                                                               |
| `lib.rs`            | The ClickHouse module exposure is tiny: **`+4 / -2`**.                                                                                                                                                                             |
| ClickHouse backend  | The real implementation mass is concentrated in the new backend files, especially `processors.rs` and `writer.rs` at **175 lines each**, plus `config.rs` at **102 lines**.                                                        |
| Jupiter integration | The decoder-specific implementation is substantial: `cpi_event_row.rs` is **280 lines** and `clickhouse/mod.rs` is **128 lines**.                                                                                                  |
| Renderer support    | The generator changes are conceptually important but patch-light in existing files: `cargoTomlGenerator.ts` is **`+19 / -2`**, `getRenderMapVisitor.ts` is **`+17 / -0`**. Most new behavior is pushed into the **new templates**. |
| Proc macros         | This should be described as a **tiny modified file with no clear semantic change**, not “identical.”                                                                                                                               |

## Interpretation

This branch is best understood as:

* **core lifecycle extension**

  * `Processor::finalize()`
  * pipe wrappers propagate it
  * `Pipeline` calls it during shutdown

* **new core ClickHouse runtime**

  * config
  * HTTP transport
  * schema executor
  * row contracts
  * buffered writer
  * ClickHouse instruction processor

* **one concrete decoder implementation**

  * Jupiter swap CPI `SwapEvent`
  * typed landing-table row
  * schema/bootstrap helpers

* **generator support**

  * ClickHouse features in generated decoder manifests
  * generated ClickHouse decoder glue and row modules
  * generated CPI-event decode path adjusted to support that cleanly

* **one proof example**

  * bounded block crawler
  * Jupiter decoder
  * ClickHouse landing-table writes

