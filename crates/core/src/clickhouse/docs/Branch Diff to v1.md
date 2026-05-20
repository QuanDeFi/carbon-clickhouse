# Branch Diff: `upstream-v1-sync` To `clickhouse-upstream-v1`

This document is a technical summary of the actual Git diff from `upstream-v1-sync` to `clickhouse-upstream-v1`. It is intentionally derived from Git commands and should be recomputed whenever either branch moves.

## Diff Basis

Commands used:

```bash
TZ=Asia/Jakarta date '+%Y-%m-%d %H:%M:%S %Z (%z)'
git fetch upstream --prune
git rev-parse --abbrev-ref HEAD
git rev-parse upstream-v1-sync
git rev-parse HEAD
git merge-base upstream-v1-sync HEAD
git rev-list --left-right --count upstream-v1-sync...HEAD
git rev-list --left-right --count upstream-v1-sync...upstream/v1.0-rc
git diff --name-status --find-renames upstream-v1-sync HEAD
git diff --numstat upstream-v1-sync HEAD
git diff --stat upstream-v1-sync HEAD
git diff --shortstat upstream-v1-sync HEAD
```

Computed values:

- Diff check timestamp: `2026-05-21 06:38:18 WIB (+0700)`
- Current branch: `clickhouse-upstream-v1`
- Base branch tip: `upstream-v1-sync` at `b79f9a2a90b71dca479d64bb588d6b095137c98d`
- Compare branch tip: `clickhouse-upstream-v1` at `6afb6a238dcdc20e933eaefeda9994d570f1d8b4`
- Merge base: `b79f9a2a90b71dca479d64bb588d6b095137c98d`
- Commit relation: `0` commits behind `upstream-v1-sync`, `37` commits ahead
- Upstream sync relation: `upstream-v1-sync` is `0` commits behind and `0` commits ahead of `upstream/v1.0-rc`
- Total changed files: `140`
- Added files: `94`
- Modified files: `46`
- Insertions: `45234`
- Deletions: `99`

Status legend:

- `[A]` / `A` = added file
- `[M]` / `M` = modified existing file

## Touched Path Tree

```text
├── .github/
│   └── workflows/
│       └── [M] check.yml
├── [M] .gitignore
├── [M] Cargo.lock
├── [M] Cargo.toml
├── [M] README.md
├── crates/
│   └── core/
│       ├── [M] Cargo.toml
│       └── src/
│           ├── [M] account.rs
│           ├── [M] account_deletion.rs
│           ├── [M] block_details.rs
│           ├── clickhouse/
│           │   ├── [A] admin.rs
│           │   ├── [A] config.rs
│           │   ├── docs/
│           │   │   ├── [A] Branch Diff to v1.md
│           │   │   ├── [A] Carbon Core v1 Architecture.md
│           │   │   ├── [A] Carbon Sink Implementation.md
│           │   │   ├── [A] ClickHouse Sink Architecture.md
│           │   │   └── [A] ClickHouse Sink Tutorial Curriculum.md
│           │   ├── [A] http.rs
│           │   ├── [A] metrics.rs
│           │   ├── [A] mod.rs
│           │   ├── [A] processors.rs
│           │   ├── rows/
│           │   │   └── [A] mod.rs
│           │   └── [A] writer.rs
│           ├── [M] instruction.rs
│           ├── [M] lib.rs
│           ├── [M] pipeline.rs
│           ├── [M] processor.rs
│           └── [M] transaction.rs
├── datasources/
│   └── rpc-gpa-datasource/
│       └── src/
│           └── [M] lib.rs
├── decoders/
│   ├── jupiter-swap-decoder/
│   │   ├── [M] Cargo.toml
│   │   └── src/
│   │       ├── accounts/
│   │       │   ├── clickhouse/
│   │       │   │   ├── [A] mod.rs
│   │       │   │   └── [A] token_ledger_row.rs
│   │       │   └── [M] mod.rs
│   │       ├── instructions/
│   │       │   ├── clickhouse/
│   │       │   │   ├── [A] best_swap_out_amount_violation_event_row.rs
│   │       │   │   ├── [A] candidate_swap_quote_error_event_row.rs
│   │       │   │   ├── [A] candidate_swap_results_event_row.rs
│   │       │   │   ├── [A] claim_row.rs
│   │       │   │   ├── [A] claim_token_row.rs
│   │       │   │   ├── [A] close_token_row.rs
│   │       │   │   ├── [A] close_wsol_token_account_row.rs
│   │       │   │   ├── [A] create_token_account_row.rs
│   │       │   │   ├── [A] create_token_ledger_row.rs
│   │       │   │   ├── [A] exact_out_route_row.rs
│   │       │   │   ├── [A] exact_out_route_v2_row.rs
│   │       │   │   ├── [A] fee_event_event_row.rs
│   │       │   │   ├── [A] mod.rs
│   │       │   │   ├── [A] route_row.rs
│   │       │   │   ├── [A] route_v2_row.rs
│   │       │   │   ├── [A] route_with_token_ledger_row.rs
│   │       │   │   ├── [A] set_token_ledger_row.rs
│   │       │   │   ├── [A] shared_accounts_exact_out_route_row.rs
│   │       │   │   ├── [A] shared_accounts_exact_out_route_v2_row.rs
│   │       │   │   ├── [A] shared_accounts_route_row.rs
│   │       │   │   ├── [A] shared_accounts_route_v2_row.rs
│   │       │   │   ├── [A] shared_accounts_route_with_token_ledger_row.rs
│   │       │   │   ├── [A] swap_event_event_row.rs
│   │       │   │   └── [A] swaps_event_event_row.rs
│   │       │   ├── [M] cpi_event.rs
│   │       │   └── [M] mod.rs
│   │       └── types/
│   │           ├── [M] candidate_swap.rs
│   │           └── [M] swap.rs
│   └── token-program-decoder/
│       ├── [M] Cargo.toml
│       └── src/
│           ├── accounts/
│           │   ├── clickhouse/
│           │   │   ├── [A] mint_row.rs
│           │   │   ├── [A] mod.rs
│           │   │   ├── [A] multisig_row.rs
│           │   │   └── [A] token_row.rs
│           │   ├── [M] mint.rs
│           │   ├── [M] mod.rs
│           │   ├── [M] multisig.rs
│           │   └── [M] token.rs
│           └── instructions/
│               ├── clickhouse/
│               │   ├── [A] amount_to_ui_amount_row.rs
│               │   ├── [A] approve_checked_row.rs
│               │   ├── [A] approve_row.rs
│               │   ├── [A] batch_row.rs
│               │   ├── [A] burn_checked_row.rs
│               │   ├── [A] burn_row.rs
│               │   ├── [A] close_account_row.rs
│               │   ├── [A] freeze_account_row.rs
│               │   ├── [A] get_account_data_size_row.rs
│               │   ├── [A] initialize_account2_row.rs
│               │   ├── [A] initialize_account3_row.rs
│               │   ├── [A] initialize_account_row.rs
│               │   ├── [A] initialize_immutable_owner_row.rs
│               │   ├── [A] initialize_mint2_row.rs
│               │   ├── [A] initialize_mint_row.rs
│               │   ├── [A] initialize_multisig2_row.rs
│               │   ├── [A] initialize_multisig_row.rs
│               │   ├── [A] mint_to_checked_row.rs
│               │   ├── [A] mint_to_row.rs
│               │   ├── [A] mod.rs
│               │   ├── [A] revoke_row.rs
│               │   ├── [A] set_authority_row.rs
│               │   ├── [A] sync_native_row.rs
│               │   ├── [A] thaw_account_row.rs
│               │   ├── [A] transfer_checked_row.rs
│               │   ├── [A] transfer_row.rs
│               │   ├── [A] ui_amount_to_amount_row.rs
│               │   ├── [A] unwrap_lamports_row.rs
│               │   └── [A] withdraw_excess_lamports_row.rs
│               └── [M] mod.rs
├── examples/
│   ├── [M] README.md
│   ├── jupiter-swap-clickhouse/
│   │   ├── [A] .env.example
│   │   ├── [A] Cargo.toml
│   │   ├── [A] README.md
│   │   └── src/
│   │       └── [A] main.rs
│   └── token-program-clickhouse/
│       ├── [A] .env.example
│       ├── [A] Cargo.toml
│       ├── [A] README.md
│       └── src/
│           └── [A] main.rs
├── metrics/
│   └── prometheus-metrics/
│       └── src/
│           └── [M] lib.rs
├── monitoring/
│   ├── [A] README.md
│   ├── [A] compose.yaml
│   ├── grafana/
│   │   ├── dashboards/
│   │   │   └── [A] carbon-clickhouse-overview.json
│   │   └── provisioning/
│   │       ├── dashboards/
│   │       │   └── [A] dashboards.yml
│   │       └── datasources/
│   │           └── [A] prometheus.yml
│   └── prometheus/
│       └── [A] prometheus.yml
├── packages/
│   ├── cli/
│   │   ├── [M] README.md
│   │   └── src/
│   │       ├── [M] cli.ts
│   │       └── lib/
│   │           ├── [M] cargoTomlGenerator.ts
│   │           ├── [M] decoder.ts
│   │           ├── [M] prompts.ts
│   │           └── [M] scaffold.ts
│   ├── renderer/
│   │   ├── [M] package.json
│   │   ├── src/
│   │   │   ├── [M] cargoTomlGenerator.ts
│   │   │   ├── [A] clickhouseDdl.ts
│   │   │   ├── [A] clickhouseRowMapper.ts
│   │   │   ├── [M] getRenderMapVisitor.ts
│   │   │   ├── [M] index.ts
│   │   │   └── utils/
│   │   │       └── [M] helpers.ts
│   │   ├── templates/
│   │   │   ├── [A] accountsClickHouseMod.njk
│   │   │   ├── [M] accountsMod.njk
│   │   │   ├── [M] accountsPage.njk
│   │   │   ├── [A] clickhouseRowPage.njk
│   │   │   ├── [A] eventInstructionClickHouseRowPage.njk
│   │   │   ├── [M] eventInstructionPage.njk
│   │   │   ├── [A] instructionsClickHouseMod.njk
│   │   │   ├── [M] instructionsMod.njk
│   │   │   └── [M] lib.njk
│   │   └── test/
│   │       └── [A] clickhouse-renderer.test.cjs
│   └── versions/
│       └── src/
│           └── [M] index.ts
└── scripts/
    └── [A] validate-clickhouse-decoder-rollout.sh
```

## Consolidated File-By-File Summary

<details>
<summary><strong>Repository root and CI</strong> - 5 files, +149 / -4</summary>

Workspace-level config, README, lockfile, and CI changes.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+2 / -2</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>.github/<wbr>workflows/<wbr>check.yml</code></td>
<td>Updates CI workflow coverage for the ClickHouse-enabled branch.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+2 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>.gitignore</code></td>
<td>Updates ignore rules for local ClickHouse and development artifacts.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+48 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>Cargo.lock</code></td>
<td>Locks dependency graph changes introduced by ClickHouse runtime, examples, and generated decoders.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+1 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>Cargo.toml</code></td>
<td>Adds workspace dependency/configuration needed by ClickHouse runtime support.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+96 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>README.md</code></td>
<td>Documents ClickHouse decoder generation, CLI options, validation flow, and example entry points.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Core pipeline lifecycle</strong> - 9 files, +63 / -0</summary>

Carbon core feature wiring and processor/pipe finalization support.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+5 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>Cargo.toml</code></td>
<td>Adds the core ClickHouse feature and optional runtime dependencies.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+5 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>account.rs</code></td>
<td>Adds pipe finalization plumbing so buffered processors can drain on shutdown.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+5 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>account_<wbr>deletion.rs</code></td>
<td>Adds pipe finalization plumbing so buffered processors can drain on shutdown.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+5 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>block_<wbr>details.rs</code></td>
<td>Adds pipe finalization plumbing so buffered processors can drain on shutdown.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+5 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>instruction.rs</code></td>
<td>Adds pipe finalization plumbing so buffered processors can drain on shutdown.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+2 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>lib.rs</code></td>
<td>Exposes the ClickHouse module from carbon-core when the feature is enabled.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+27 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>pipeline.rs</code></td>
<td>Adds pipeline shutdown finalization so pipes drain before exporter shutdown completes.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+4 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>processor.rs</code></td>
<td>Adds the default processor finalize lifecycle hook.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+5 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>transaction.rs</code></td>
<td>Adds pipe finalization plumbing so buffered processors can drain on shutdown.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Core ClickHouse runtime</strong> - 8 files, +3637 / -0</summary>

Runtime config, HTTP/admin client, writer, processors, metrics, rows, and schema reconciliation.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>A</code></td>
<td><code>+735 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>admin.rs</code></td>
<td>Adds ClickHouse schema/admin execution, managed table metadata reconciliation, live schema inspection, safe enum-extension repair, and drift rejection.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+379 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>config.rs</code></td>
<td>Adds ClickHouse connection config, insert settings, batching, transport, retry, deduplication, and row context configuration.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+243 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>http.rs</code></td>
<td>Adds authenticated HTTP query/insert helpers, query settings, gzip support, query IDs, dedup tokens, metadata reads, and error classification.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+285 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>metrics.rs</code></td>
<td>Adds aggregate ClickHouse sink metrics for account and instruction processor families.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+22 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Exposes ClickHouse runtime modules and public types behind the feature gate.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+400 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>processors.rs</code></td>
<td>Adds ClickHouse account and instruction processors with buffering, finalization, and metrics integration.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+109 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>rows/<wbr>mod.rs</code></td>
<td>Adds row/table traits, row context, deterministic IDs, partition helpers, and multi-row emission contracts.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1464 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>writer.rs</code></td>
<td>Adds per-table/per-partition buffered writer, byte accounting, backpressure, background flushing, retries, dedup tokens, snapshots, shutdown drain, and tests.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Core ClickHouse docs</strong> - 5 files, +2633 / -0</summary>

ClickHouse architecture, implementation, tutorial, and branch-diff documentation.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>A</code></td>
<td><code>+261 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>docs/<wbr>Branch Diff to v1.md</code></td>
<td>This computed branch-diff document.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+750 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>docs/<wbr>Carbon Core v1 Architecture.md</code></td>
<td>Documents Carbon v1 architecture and upstream merge-risk context.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+884 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>docs/<wbr>Carbon Sink Implementation.md</code></td>
<td>Documents the current ClickHouse sink implementation, runtime APIs, generated code, reliability behavior, and operations.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+307 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>docs/<wbr>ClickHouse Sink Architecture.md</code></td>
<td>Documents ClickHouse sink architecture, responsibility split, invariants, and roadmap.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+431 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>docs/<wbr>ClickHouse Sink Tutorial Curriculum.md</code></td>
<td>Adds a tutorial curriculum for explaining and validating the ClickHouse sink.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>RPC GPA datasource</strong> - 1 file, +13 / -5</summary>

Bounded account datasource behavior and error reporting.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+13 / -5</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>datasources/<wbr>rpc-<wbr>gpa-<wbr>datasource/<wbr>src/<wbr>lib.rs</code></td>
<td>Improves bounded GPA error handling and reporting for RPC/account smoke tests.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Jupiter Swap decoder</strong> - 32 files, +20101 / -1</summary>

Generated ClickHouse account, instruction, and CPI/event landing support for the Jupiter canary.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+8 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>Cargo.toml</code></td>
<td>Adds Jupiter ClickHouse feature wiring and optional dependencies.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+149 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Adds generated Jupiter TokenLedger account ClickHouse landing row dispatch, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+379 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Adds generated Jupiter TokenLedger account ClickHouse landing row dispatch, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+3 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>mod.rs</code></td>
<td>Exposes generated Jupiter account ClickHouse module behind the feature gate.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+407 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>best_<wbr>swap_<wbr>out_<wbr>amount_<wbr>violation_<wbr>event_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+417 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>candidate_<wbr>swap_<wbr>quote_<wbr>error_<wbr>event_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+443 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>candidate_<wbr>swap_<wbr>results_<wbr>event_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+384 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>claim_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+384 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>claim_<wbr>token_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+395 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>close_<wbr>token_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+374 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>close_<wbr>wsol_<wbr>token_<wbr>account_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+385 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>create_<wbr>token_<wbr>account_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+374 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>create_<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1362 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>exact_<wbr>out_<wbr>route_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1374 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>exact_<wbr>out_<wbr>route_<wbr>v2_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+417 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>fee_<wbr>event_<wbr>event_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+592 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Adds generated Jupiter instruction/CPI-event ClickHouse dispatch, migrations, setup helpers, processor alias, and explicit public exports.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1362 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>route_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1373 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>route_<wbr>v2_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1352 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>route_<wbr>with_<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+374 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>set_<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1375 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>exact_<wbr>out_<wbr>route_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1386 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>exact_<wbr>out_<wbr>route_<wbr>v2_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1374 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>route_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1385 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>route_<wbr>v2_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1364 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>route_<wbr>with_<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+439 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>swap_<wbr>event_<wbr>event_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+429 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>swaps_<wbr>event_<wbr>event_<wbr>row.rs</code></td>
<td>Adds generated Jupiter instruction or CPI/event ClickHouse landing row, structured payload mapping, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+19 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>cpi_<wbr>event.rs</code></td>
<td>Adds CPI-event account construction support for decoded Jupiter event instructions.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+14 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>mod.rs</code></td>
<td>Exposes generated Jupiter ClickHouse instruction module and CPI-event decode path.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+1 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>types/<wbr>candidate_<wbr>swap.rs</code></td>
<td>Updates generated Jupiter shared types used by structured ClickHouse row conversion.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+7 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>jupiter-<wbr>swap-<wbr>decoder/<wbr>src/<wbr>types/<wbr>swap.rs</code></td>
<td>Updates generated Jupiter shared types used by structured ClickHouse row conversion.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Token Program decoder</strong> - 39 files, +13122 / -63</summary>

Generated ClickHouse account and instruction landing support plus SPL token account decoding fixes.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+11 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>Cargo.toml</code></td>
<td>Adds Token Program ClickHouse feature wiring and optional dependencies.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+418 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>mint_<wbr>row.rs</code></td>
<td>Adds generated Token Program account ClickHouse landing row, DDL, conversion, SPL-token unpacking support, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+192 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Adds generated Token Program account ClickHouse dispatch, migrations, setup helpers, processor alias, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+405 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>multisig_<wbr>row.rs</code></td>
<td>Adds generated Token Program account ClickHouse landing row, DDL, conversion, SPL-token unpacking support, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+448 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>token_<wbr>row.rs</code></td>
<td>Adds generated Token Program account ClickHouse landing row, DDL, conversion, SPL-token unpacking support, and managed schema metadata.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+8 / -10</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>mint.rs</code></td>
<td>Updates generated Token Program account decoding to align with official SPL token account layouts and ClickHouse output.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+79 / -33</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>mod.rs</code></td>
<td>Updates generated Token Program account decoding to align with official SPL token account layouts and ClickHouse output.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+7 / -10</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>multisig.rs</code></td>
<td>Updates generated Token Program account decoding to align with official SPL token account layouts and ClickHouse output.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+21 / -10</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>accounts/<wbr>token.rs</code></td>
<td>Updates generated Token Program account decoding to align with official SPL token account layouts and ClickHouse output.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+385 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>amount_<wbr>to_<wbr>ui_<wbr>amount_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+396 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>approve_<wbr>checked_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+384 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>approve_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+412 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>batch_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+395 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>burn_<wbr>checked_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+384 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>burn_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+373 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>close_<wbr>account_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+373 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>freeze_<wbr>account_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+374 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>get_<wbr>account_<wbr>data_<wbr>size_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+385 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>account2_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+385 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>account3_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+374 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>account_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+374 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>immutable_<wbr>owner_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+410 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>mint2_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+410 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>mint_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+385 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>multisig2_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+385 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>multisig_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+396 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>mint_<wbr>to_<wbr>checked_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+384 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>mint_<wbr>to_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+728 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse dispatch, migrations, setup helpers, processor alias, and explicit public exports.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+373 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>revoke_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+395 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>set_<wbr>authority_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+373 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>sync_<wbr>native_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+373 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>thaw_<wbr>account_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+396 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>transfer_<wbr>checked_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+384 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>transfer_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+385 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>ui_<wbr>amount_<wbr>to_<wbr>amount_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+385 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>unwrap_<wbr>lamports_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+374 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>withdraw_<wbr>excess_<wbr>lamports_<wbr>row.rs</code></td>
<td>Adds generated Token Program instruction ClickHouse landing row, DDL, conversion, and managed schema metadata.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+3 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>decoders/<wbr>token-<wbr>program-<wbr>decoder/<wbr>src/<wbr>instructions/<wbr>mod.rs</code></td>
<td>Exposes generated Token Program instruction ClickHouse module behind the feature gate.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Examples</strong> - 9 files, +971 / -0</summary>

Jupiter and Token Program ClickHouse smoke examples, env files, and example docs.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+1 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>examples/<wbr>README.md</code></td>
<td>Adds ClickHouse examples to the workspace example index.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+22 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>examples/<wbr>jupiter-<wbr>swap-<wbr>clickhouse/<wbr>.env.example</code></td>
<td>Documents Jupiter ClickHouse example environment variables.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+21 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>examples/<wbr>jupiter-<wbr>swap-<wbr>clickhouse/<wbr>Cargo.toml</code></td>
<td>Adds the Jupiter Swap ClickHouse example crate and dependencies.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+242 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>examples/<wbr>jupiter-<wbr>swap-<wbr>clickhouse/<wbr>README.md</code></td>
<td>Documents Jupiter ClickHouse example behavior, generated landing tables, run modes, and validation queries.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+248 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>examples/<wbr>jupiter-<wbr>swap-<wbr>clickhouse/<wbr>src/<wbr>main.rs</code></td>
<td>Adds Jupiter bounded/head-following ClickHouse smoke example with instruction, CPI/event, and live TokenLedger account paths.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+32 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>examples/<wbr>token-<wbr>program-<wbr>clickhouse/<wbr>.env.example</code></td>
<td>Documents Token Program ClickHouse example environment variables.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+23 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>examples/<wbr>token-<wbr>program-<wbr>clickhouse/<wbr>Cargo.toml</code></td>
<td>Adds the Token Program ClickHouse example crate and dependencies.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+127 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>examples/<wbr>token-<wbr>program-<wbr>clickhouse/<wbr>README.md</code></td>
<td>Documents Token Program ClickHouse account example behavior, table purpose, provider notes, and validation queries.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+255 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>examples/<wbr>token-<wbr>program-<wbr>clickhouse/<wbr>src/<wbr>main.rs</code></td>
<td>Adds Token Program account snapshot ClickHouse smoke example using RPC/GPA-derived account data.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Metrics exporter</strong> - 1 file, +22 / -1</summary>

Prometheus metric export compatibility for Carbon/ClickHouse metric names.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+22 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>metrics/<wbr>prometheus-<wbr>metrics/<wbr>src/<wbr>lib.rs</code></td>
<td>Adjusts Prometheus export normalization for Carbon and ClickHouse metric names.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Monitoring stack</strong> - 6 files, +664 / -0</summary>

Local Prometheus/Grafana compose, provisioning, and dashboard assets.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>A</code></td>
<td><code>+101 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>monitoring/<wbr>README.md</code></td>
<td>Documents local Prometheus/Grafana setup for Carbon ClickHouse metrics.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+37 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>monitoring/<wbr>compose.yaml</code></td>
<td>Adds local Prometheus/Grafana compose stack.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+490 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>monitoring/<wbr>grafana/<wbr>dashboards/<wbr>carbon-<wbr>clickhouse-<wbr>overview.json</code></td>
<td>Adds Grafana dashboard for Carbon process and ClickHouse sink metrics.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+11 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>monitoring/<wbr>grafana/<wbr>provisioning/<wbr>dashboards/<wbr>dashboards.yml</code></td>
<td>Adds Prometheus/Grafana provisioning for the local monitoring stack.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+10 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>monitoring/<wbr>grafana/<wbr>provisioning/<wbr>datasources/<wbr>prometheus.yml</code></td>
<td>Adds Prometheus/Grafana provisioning for the local monitoring stack.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+15 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>monitoring/<wbr>prometheus/<wbr>prometheus.yml</code></td>
<td>Adds Prometheus/Grafana provisioning for the local monitoring stack.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>CLI</strong> - 6 files, +109 / -6</summary>

ClickHouse generation flags, prompt/config plumbing, and decoder scaffolding support.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+14 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>cli/<wbr>README.md</code></td>
<td>Documents CLI ClickHouse generation options.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+70 / -4</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>cli/<wbr>src/<wbr>cli.ts</code></td>
<td>Threads ClickHouse renderer options and generated dependency wiring through the decoder CLI.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+10 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>cli/<wbr>src/<wbr>lib/<wbr>cargoTomlGenerator.ts</code></td>
<td>Threads ClickHouse renderer options and generated dependency wiring through the decoder CLI.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+10 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>cli/<wbr>src/<wbr>lib/<wbr>decoder.ts</code></td>
<td>Threads ClickHouse renderer options and generated dependency wiring through the decoder CLI.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+2 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>cli/<wbr>src/<wbr>lib/<wbr>prompts.ts</code></td>
<td>Threads ClickHouse renderer options and generated dependency wiring through the decoder CLI.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+3 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>cli/<wbr>src/<wbr>lib/<wbr>scaffold.ts</code></td>
<td>Threads ClickHouse renderer options and generated dependency wiring through the decoder CLI.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Renderer</strong> - 17 files, +3397 / -19</summary>

ClickHouse schema mapper, DDL planner, templates, and renderer tests.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+1 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>package.json</code></td>
<td>Adds renderer test/type-check support for ClickHouse generation.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+43 / -3</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>src/<wbr>cargoTomlGenerator.ts</code></td>
<td>Adds renderer-side Cargo manifest generation for ClickHouse features and dependencies.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+196 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>src/<wbr>clickhouseDdl.ts</code></td>
<td>Adds renderer-controlled ClickHouse DDL planning for MergeTree, replicated, distributed, cluster, TTL, key, codec, and additive migration options.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+1094 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>src/<wbr>clickhouseRowMapper.ts</code></td>
<td>Adds strict structured ClickHouse schema planning from Codama types, including primitives, composites, arrays, enums, payload unions, and fallback control.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+163 / -4</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>src/<wbr>getRenderMapVisitor.ts</code></td>
<td>Hooks ClickHouse generation into account, instruction, and CPI/event rendering with DDL options and fallback handling.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+4 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>src/<wbr>index.ts</code></td>
<td>Exports ClickHouse renderer planning APIs.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+13 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>src/<wbr>utils/<wbr>helpers.ts</code></td>
<td>Adds renderer helper support used by generated ClickHouse modules.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+161 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>templates/<wbr>accountsClickHouseMod.njk</code></td>
<td>Adds generated ClickHouse module dispatch, migrations, setup helpers, managed table metadata, and public exports.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+94 / -3</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>templates/<wbr>accountsMod.njk</code></td>
<td>Updates existing renderer templates to include feature-gated ClickHouse modules and CPI/event support.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+52 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>templates/<wbr>accountsPage.njk</code></td>
<td>Updates existing renderer templates to include feature-gated ClickHouse modules and CPI/event support.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+474 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>templates/<wbr>clickhouseRowPage.njk</code></td>
<td>Adds generated ClickHouse row templates with typed row structs, conversions, DDL, column metadata, and managed schema helpers.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+344 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>templates/<wbr>eventInstructionClickHouseRowPage.njk</code></td>
<td>Adds generated ClickHouse row templates with typed row structs, conversions, DDL, column metadata, and managed schema helpers.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+17 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>templates/<wbr>eventInstructionPage.njk</code></td>
<td>Updates existing renderer templates to include feature-gated ClickHouse modules and CPI/event support.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+220 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>templates/<wbr>instructionsClickHouseMod.njk</code></td>
<td>Adds generated ClickHouse module dispatch, migrations, setup helpers, managed table metadata, and public exports.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+20 / -4</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>templates/<wbr>instructionsMod.njk</code></td>
<td>Updates existing renderer templates to include feature-gated ClickHouse modules and CPI/event support.</td>
</tr>
<tr>
<td><code>M</code></td>
<td><code>+1 / -1</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>templates/<wbr>lib.njk</code></td>
<td>Updates existing renderer templates to include feature-gated ClickHouse modules and CPI/event support.</td>
</tr>
<tr>
<td><code>A</code></td>
<td><code>+500 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>renderer/<wbr>test/<wbr>clickhouse-<wbr>renderer.test.cjs</code></td>
<td>Adds renderer tests for ClickHouse rows, DDL modes, strict fallback behavior, managed metadata, and generated modules.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Version metadata</strong> - 1 file, +1 / -0</summary>

Package version metadata for the ClickHouse-enabled generation path.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>M</code></td>
<td><code>+1 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>packages/<wbr>versions/<wbr>src/<wbr>index.ts</code></td>
<td>Updates package version metadata for ClickHouse-capable generation.</td>
</tr>
</tbody>
</table>
</div>

</details>

<details>
<summary><strong>Rollout scripts</strong> - 1 file, +352 / -0</summary>

Non-committing generated-decoder rollout validation tooling.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 4.5rem;" />
<col style="width: 7rem;" />
<col style="width: 34%;" />
<col style="width: auto;" />
</colgroup>
<thead>
<tr>
<th>Status</th>
<th>+/-</th>
<th>File</th>
<th>Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>A</code></td>
<td><code>+352 / -0</code></td>
<td style="overflow-wrap: anywhere; word-break: break-word;"><code>scripts/<wbr>validate-<wbr>clickhouse-<wbr>decoder-<wbr>rollout.sh</code></td>
<td>Adds non-committing ClickHouse decoder rollout validation script for canary and broad generated-decoder checks.</td>
</tr>
</tbody>
</table>
</div>

</details>
