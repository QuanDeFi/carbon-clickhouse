# ClickHouse vs Postgres Sink Diff

This document is a technical comparison of the local ClickHouse sink implementation against the local Postgres sink implementation. It is not a branch diff: it compares sink-related paths that exist in the current worktree.

## Diff Basis

Commands used:

```bash
TZ=Asia/Jakarta date '+%Y-%m-%d %H:%M:%S %Z (%z)'
git rev-parse --abbrev-ref HEAD
git rev-parse HEAD
git status --short
find crates/core/src/clickhouse -type f ! -path '*/docs/*'
find crates/core/src/postgres -type f
find packages/renderer -maxdepth 3 \( -iname '*clickhouse*' -o -iname '*postgres*' \) -type f
find decoders/jupiter-swap-decoder/src -path '*/clickhouse/*' -o -path '*/postgres/*'
find decoders/token-program-decoder/src -path '*/clickhouse/*' -o -path '*/postgres/*'
find examples -maxdepth 2 \( -iname '*clickhouse*' -o -iname '*postgres*' \) -print
```

Line counts were computed from the local worktree with an inline Node.js script that counts files and physical text lines for the scoped paths below. The calculation includes scoped untracked files in the dirty worktree.

Computed values:

- Diff check timestamp: `2026-05-22 12:25:17 WIB (+0700)`
- Current branch: `clickhouse-upstream-v1`
- Current commit: `08f424a4b01db1fbbb3be361f09052b0148446a1`
- Comparison type: local worktree sink-family comparison, not `git diff`
- ClickHouse scoped files: `85`
- Postgres scoped files: `72`
- ClickHouse scoped lines: `17,006`
- Postgres scoped lines: `14,566`
- ClickHouse/Postgres scoped LOC ratio: `1.17x`

Scope notes:

- Core runtime compares `crates/core/src/clickhouse` excluding docs against `crates/core/src/postgres`.
- Renderer compares ClickHouse mapper/DDL/templates/tests against Postgres mapper/templates/helpers.
- Generated decoder output compares committed ClickHouse canaries only: Jupiter Swap and Token Program.
- Postgres has broad generated output across many more decoders. This document intentionally does not compare the full Postgres decoder universe against the canary-limited ClickHouse rollout.
- Example scope excludes local `.env` files and compares committed example source/docs/config only.

## Local Sink Footprint

<strong>Scoped ClickHouse vs Postgres line footprint</strong>

<div style="display: flex; width: 100%; height: 24px; overflow: hidden;">
  <div title="ClickHouse scoped lines: 17,006 lines, 53.9% of combined scoped LOC" style="width: 53.9%; background: #4F684E; color: #F8FAFC; line-height: 24px;">&nbsp;ClickHouse 53.9%</div>
  <div title="Postgres scoped lines: 14,566 lines, 46.1% of combined scoped LOC" style="width: 46.1%; background: #5A4F69; color: #F8FAFC; line-height: 24px;">&nbsp;Postgres 46.1%</div>
</div>

<table width="100%">
<tr>
<td><font color="#4F684E">■</font> ClickHouse: <strong>17,006</strong> lines / <strong>85</strong> files</td>
<td><font color="#5A4F69">■</font> Postgres: <strong>14,566</strong> lines / <strong>72</strong> files</td>
<td align="right">Combined scoped LOC: <strong>31,572</strong></td>
</tr>
</table>

<strong>Section footprint</strong>

<table width="100%" cellspacing="0" cellpadding="6">
<thead>
<tr>
<th align="left" width="22%">Section</th>
<th align="left" width="46%">Visual</th>
<th align="right" width="16%">ClickHouse</th>
<th align="right" width="16%">Postgres</th>
</tr>
</thead>
<tbody>
<tr>
<td>Core runtime</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="ClickHouse: 4,173 lines" style="width: 13.2%; background: #4F684E;">&nbsp;</div>
  <div title="Postgres: 961 lines" style="width: 3.0%; background: #5A4F69;">&nbsp;</div>
  <div style="width: 83.8%;">&nbsp;</div>
</div>
5,134 combined lines
</td>
<td align="right" nowrap>8 files / 4,173 lines</td>
<td align="right" nowrap>6 files / 961 lines</td>
</tr>
<tr>
<td>Renderer</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="ClickHouse: 2,539 lines" style="width: 8.0%; background: #6B5B45;">&nbsp;</div>
  <div title="Postgres: 1,313 lines" style="width: 4.2%; background: #5A4F69;">&nbsp;</div>
  <div style="width: 87.8%;">&nbsp;</div>
</div>
3,852 combined lines
</td>
<td align="right" nowrap>8 files / 2,539 lines</td>
<td align="right" nowrap>6 files / 1,313 lines</td>
</tr>
<tr>
<td>Jupiter Swap canary</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="ClickHouse: 4,860 lines" style="width: 15.4%; background: #435A6F;">&nbsp;</div>
  <div title="Postgres: 4,921 lines" style="width: 15.6%; background: #5A4F69;">&nbsp;</div>
  <div style="width: 69.0%;">&nbsp;</div>
</div>
9,781 combined lines
</td>
<td align="right" nowrap>27 files / 4,860 lines</td>
<td align="right" nowrap>21 files / 4,921 lines</td>
</tr>
<tr>
<td>Token Program canary</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="ClickHouse: 4,618 lines" style="width: 14.6%; background: #435A6F;">&nbsp;</div>
  <div title="Postgres: 7,037 lines" style="width: 22.3%; background: #5A4F69;">&nbsp;</div>
  <div style="width: 63.1%;">&nbsp;</div>
</div>
11,655 combined lines
</td>
<td align="right" nowrap>34 files / 4,618 lines</td>
<td align="right" nowrap>33 files / 7,037 lines</td>
</tr>
<tr>
<td>Examples</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="ClickHouse: 816 lines" style="width: 2.6%; background: #52645E;">&nbsp;</div>
  <div title="Postgres: 334 lines" style="width: 1.1%; background: #5A4F69;">&nbsp;</div>
  <div style="width: 96.3%;">&nbsp;</div>
</div>
1,150 combined lines
</td>
<td align="right" nowrap>8 files / 816 lines</td>
<td align="right" nowrap>6 files / 334 lines</td>
</tr>
</tbody>
</table>

The current dirty worktree reflects the generated ClickHouse helper deduplication pass. ClickHouse still has a larger core runtime because it owns HTTP transport, batching, background flushing, retry/backoff, dedup query settings, schema drift reconciliation, ClickHouse DDL generation, and typed landing-table contracts. Generated canary output is now much closer to Postgres: Jupiter ClickHouse is slightly smaller than Jupiter Postgres by physical LOC, and Token Program ClickHouse is materially smaller than Token Program Postgres.

## Consolidated Comparison Summary

<details open>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Core runtime</strong></span><span style="display: inline-block; width: 22%; text-align: left;"><code>CH 8 / PG 6 files</code></span><span style="display: inline-block; width: 33%; text-align: right;"><code>CH 4173 / PG 961 lines</code></span></summary>

ClickHouse is a buffered append-only landing sink. Postgres is an immediate `sqlx` upsert sink with generic JSON rows and typed generated rows.

<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 24%;" />
<col style="width: 24%;" />
<col style="width: 52%;" />
</colgroup>
<thead>
<tr>
<th align="left">ClickHouse path</th>
<th align="left">Postgres counterpart</th>
<th align="left">Comparison</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>admin.rs</code></td>
<td><code>rows.rs</code> migrations</td>
<td>ClickHouse adds admin execution, compact column-spec helpers, managed table metadata, layout checks, column introspection, missing-column adds, and safe enum-extension drift repair. Postgres row migrations use `sqlx_migrator` to create/drop generic tables.</td>
</tr>
<tr>
<td><code>config.rs</code></td>
<td>none</td>
<td>ClickHouse owns explicit sink configuration: endpoint/auth, sync vs async-wait insert settings, row/byte batching, global buffer caps, transport knobs, retry/backoff, and exact-batch dedup tokens. Postgres relies on `PgPool` configuration outside the sink module.</td>
</tr>
<tr>
<td><code>http.rs</code></td>
<td>none</td>
<td>ClickHouse uses HTTP query/data inserts, optional gzip, auth, query settings, response parsing, and retryable/permanent error classification. Postgres uses direct `sqlx` queries.</td>
</tr>
<tr>
<td><code>metrics.rs</code></td>
<td><code>processors.rs</code> statics</td>
<td>ClickHouse centralizes inserted/failed rows, bytes, retries, backpressure, buffer gauges, and flush duration metrics by account/instruction family. Postgres records upsert success/failure and duration counters/histograms in processor code.</td>
</tr>
<tr>
<td><code>processors.rs</code></td>
<td><code>processors.rs</code></td>
<td>Both expose account and instruction processors. ClickHouse converts decoded inputs into one or more landing rows and buffers them until flush/finalize. Postgres converts decoded inputs into one row and immediately upserts it.</td>
</tr>
<tr>
<td><code>rows/mod.rs</code></td>
<td><code>metadata.rs</code>, <code>primitives.rs</code>, <code>rows.rs</code></td>
<td>ClickHouse defines row/table contracts, row context, common landing metadata structs, deterministic IDs, typed multi-row emission, and JSONEachRow serialization. Postgres defines SQL metadata wrappers, primitive adapters, generic JSON row structs, and CRUD traits.</td>
</tr>
<tr>
<td><code>writer.rs</code></td>
<td>none</td>
<td>ClickHouse adds per-table/per-partition buffers, row and byte thresholds, background stale flushing, global backpressure, retry loops, query IDs, optional dedup tokens, failed-buffer reinsertion, snapshots, and shutdown drain.</td>
</tr>
</tbody>
</table>

</details>

<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Renderer</strong></span><span style="display: inline-block; width: 22%; text-align: left;"><code>CH 8 / PG 6 files</code></span><span style="display: inline-block; width: 33%; text-align: right;"><code>CH 2539 / PG 1313 lines</code></span></summary>

The renderer is where the two sinks diverge most semantically. Postgres emits SQL row/upsert code. ClickHouse emits typed landing rows plus DDL, managed table metadata, schema drift metadata, production DDL options, and shared helper modules.

<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 24%;" />
<col style="width: 24%;" />
<col style="width: 52%;" />
</colgroup>
<thead>
<tr>
<th align="left">ClickHouse path</th>
<th align="left">Postgres counterpart</th>
<th align="left">Comparison</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>clickhouseDdl.ts</code></td>
<td>none</td>
<td>ClickHouse owns generated DDL modes and table options: MergeTree, ReplicatedMergeTree, Distributed, `ON CLUSTER`, partition/order expressions, TTL, engine settings, codecs, and additive column migration SQL.</td>
</tr>
<tr>
<td><code>clickhouseRowMapper.ts</code></td>
<td><code>postgresRowMapper.ts</code></td>
<td>ClickHouse maps Codama schema into ClickHouse scalar, array, tuple, enum, tagged-union, wrapper, conversion, DDL, and import plans. Postgres maps the same decoder schema into SQL column types and row expressions.</td>
</tr>
<tr>
<td><code>clickhouseTypesPage.njk</code></td>
<td>none</td>
<td>ClickHouse now emits shared helper structs/functions once per generated ClickHouse module instead of repeating large helper types in every row file.</td>
</tr>
<tr>
<td><code>accountsClickHouseMod.njk</code></td>
<td><code>accountsPostgresMod.njk</code></td>
<td>Both generate account modules and processor aliases. ClickHouse also emits setup helpers, managed tables, explicit exported row types, and optional shared helper modules for landing tables.</td>
</tr>
<tr>
<td><code>instructionsClickHouseMod.njk</code></td>
<td><code>instructionsPostgresMod.njk</code></td>
<td>Both dispatch generated instruction rows. ClickHouse also dispatches generated per-event CPI/event rows and exposes managed table metadata for each row family.</td>
</tr>
<tr>
<td><code>clickhouseRowPage.njk</code>, <code>eventInstructionClickHouseRowPage.njk</code></td>
<td><code>postgresRowPage.njk</code></td>
<td>ClickHouse row templates now emit compact row structs, conversions, payload column specs, and calls into core DDL/metadata builders. Postgres emits `sqlx` row structs and upsert SQL.</td>
</tr>
<tr>
<td><code>clickhouse-renderer.test.cjs</code></td>
<td>renderer Postgres helper coverage</td>
<td>ClickHouse has dedicated tests for structured type mapping, DDL options, shared helper generation, generated module output, fallback behavior, and canary generation safety.</td>
</tr>
</tbody>
</table>

</details>

<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Jupiter Swap canary</strong></span><span style="display: inline-block; width: 22%; text-align: left;"><code>CH 27 / PG 21 files</code></span><span style="display: inline-block; width: 33%; text-align: right;"><code>CH 4860 / PG 4921 lines</code></span></summary>

Jupiter is the high-complexity instruction and CPI/event canary. After shared helper extraction, large route-plan helpers live in `instructions/clickhouse/types.rs`, and individual ClickHouse row files are now compact.

<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 24%;" />
<col style="width: 24%;" />
<col style="width: 52%;" />
</colgroup>
<thead>
<tr>
<th align="left">Surface</th>
<th align="left">Counts</th>
<th align="left">Comparison</th>
</tr>
</thead>
<tbody>
<tr>
<td>Accounts</td>
<td><code>CH 1 row / PG 1 row</code></td>
<td>Both generate the TokenLedger account row. ClickHouse adds account landing table metadata and append-only landing context, but the row file uses shared core DDL helpers.</td>
</tr>
<tr>
<td>Normal instructions</td>
<td><code>CH 17 rows / PG 17 rows</code></td>
<td>The normal Jupiter instruction row families match Postgres by name. ClickHouse route rows use structured ClickHouse tuples/enums/arrays and import shared helper types instead of repeating those helpers per row.</td>
</tr>
<tr>
<td>CPI/events</td>
<td><code>CH 6 rows / PG 1 row</code></td>
<td>Postgres uses one generated `cpi_event_row.rs`. ClickHouse currently uses separate typed event landing tables for fee, swap, swaps, candidate swap results, candidate quote errors, and best-out violation events.</td>
</tr>
<tr>
<td>Shared helpers</td>
<td><code>CH types.rs 1037 lines / PG none</code></td>
<td>ClickHouse centralizes Jupiter route-plan, candidate-swap, enum-wrapper, tuple, and large integer helper types in one module. This is the primary reason the individual row files are now much smaller.</td>
</tr>
<tr>
<td>Module dispatch</td>
<td><code>CH mod.rs 593 lines / PG mod.rs 421 lines</code></td>
<td>ClickHouse dispatches normal instruction rows and per-event rows, exports row/processor/migration helpers explicitly, includes the shared `types` module, and provides managed table setup helpers.</td>
</tr>
</tbody>
</table>

</details>

<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Token Program canary</strong></span><span style="display: inline-block; width: 22%; text-align: left;"><code>CH 34 / PG 33 files</code></span><span style="display: inline-block; width: 33%; text-align: right;"><code>CH 4618 / PG 7037 lines</code></span></summary>

Token Program is the account-family canary. ClickHouse and Postgres expose the same generated account and instruction row families, but the ClickHouse row files are now shorter because common landing metadata and DDL generation moved into core helpers.

<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 24%;" />
<col style="width: 24%;" />
<col style="width: 52%;" />
</colgroup>
<thead>
<tr>
<th align="left">Surface</th>
<th align="left">Counts</th>
<th align="left">Comparison</th>
</tr>
</thead>
<tbody>
<tr>
<td>Accounts</td>
<td><code>CH 3 rows / PG 3 rows</code></td>
<td>Both generate Mint, Multisig, and Token account rows. ClickHouse uses structured landing columns and SPL Token unpacked account values; Postgres uses generated SQL rows.</td>
</tr>
<tr>
<td>Instructions</td>
<td><code>CH 28 rows / PG 28 rows</code></td>
<td>The instruction row families match Postgres by name. ClickHouse emits compact payload specs and reuses core metadata/DDL helpers instead of embedding full DDL functions in every row file.</td>
</tr>
<tr>
<td>Shared helpers</td>
<td><code>CH types.rs 27 lines / PG none</code></td>
<td>Token Program currently needs only a small shared ClickHouse helper module, but the structure matches Jupiter and keeps future helper growth out of individual row files.</td>
</tr>
<tr>
<td>Module dispatch</td>
<td><code>CH mod.rs 729 lines / PG mod.rs 636 lines</code></td>
<td>ClickHouse has the same instruction dispatch surface plus generated setup, managed schema helpers, and the shared `types` module.</td>
</tr>
</tbody>
</table>

</details>

<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Examples</strong></span><span style="display: inline-block; width: 22%; text-align: left;"><code>CH 8 / PG 6 files</code></span><span style="display: inline-block; width: 33%; text-align: right;"><code>CH 816 / PG 334 lines</code></span></summary>

The examples are not one-to-one equivalents. ClickHouse has two real-world sink canaries. Postgres has a Postgres/GraphQL example.

<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 24%;" />
<col style="width: 24%;" />
<col style="width: 52%;" />
</colgroup>
<thead>
<tr>
<th align="left">ClickHouse examples</th>
<th align="left">Postgres example</th>
<th align="left">Comparison</th>
</tr>
</thead>
<tbody>
<tr>
<td><code>examples/jupiter-swap-clickhouse</code></td>
<td><code>examples/postgres-graphql</code></td>
<td>Jupiter ClickHouse exercises real block-crawler ingestion, table bootstrap, instruction/event landing rows, optional live TokenLedger account fetching, metrics, and local ClickHouse insertion.</td>
</tr>
<tr>
<td><code>examples/token-program-clickhouse</code></td>
<td><code>examples/postgres-graphql</code></td>
<td>Token Program ClickHouse exercises account landing rows for USDC token, mint, and multisig snapshots. The Postgres example demonstrates Postgres-backed GraphQL rather than a parallel token/Jupiter sink canary.</td>
</tr>
</tbody>
</table>

</details>

## Exact File Inventory

<details>
<summary><strong>Core runtime file counts</strong></summary>

| Side | File | Lines |
| --- | --- | ---: |
| CH | `crates/core/src/clickhouse/admin.rs` | 957 |
| CH | `crates/core/src/clickhouse/config.rs` | 379 |
| CH | `crates/core/src/clickhouse/http.rs` | 243 |
| CH | `crates/core/src/clickhouse/metrics.rs` | 285 |
| CH | `crates/core/src/clickhouse/mod.rs` | 25 |
| CH | `crates/core/src/clickhouse/processors.rs` | 400 |
| CH | `crates/core/src/clickhouse/rows/mod.rs` | 420 |
| CH | `crates/core/src/clickhouse/writer.rs` | 1464 |
| PG | `crates/core/src/postgres/metadata.rs` | 48 |
| PG | `crates/core/src/postgres/mod.rs` | 20 |
| PG | `crates/core/src/postgres/operations.rs` | 38 |
| PG | `crates/core/src/postgres/primitives.rs` | 309 |
| PG | `crates/core/src/postgres/processors.rs` | 236 |
| PG | `crates/core/src/postgres/rows.rs` | 310 |

</details>

<details>
<summary><strong>Renderer file counts</strong></summary>

| Side | File | Lines |
| --- | --- | ---: |
| CH | `packages/renderer/src/clickhouseDdl.ts` | 196 |
| CH | `packages/renderer/src/clickhouseRowMapper.ts` | 1094 |
| CH | `packages/renderer/templates/accountsClickHouseMod.njk` | 164 |
| CH | `packages/renderer/templates/clickhouseRowPage.njk` | 183 |
| CH | `packages/renderer/templates/clickhouseTypesPage.njk` | 11 |
| CH | `packages/renderer/templates/eventInstructionClickHouseRowPage.njk` | 141 |
| CH | `packages/renderer/templates/instructionsClickHouseMod.njk` | 223 |
| CH | `packages/renderer/test/clickhouse-renderer.test.cjs` | 527 |
| PG | `packages/renderer/src/getPostgresTypeManifestVisitor.ts` | 167 |
| PG | `packages/renderer/src/postgresRowMapper.ts` | 562 |
| PG | `packages/renderer/src/utils/postgresHelpers.ts` | 126 |
| PG | `packages/renderer/templates/accountsPostgresMod.njk` | 88 |
| PG | `packages/renderer/templates/instructionsPostgresMod.njk` | 102 |
| PG | `packages/renderer/templates/postgresRowPage.njk` | 268 |

</details>

<details>
<summary><strong>Canary generated output counts</strong></summary>

| Decoder | Side | Files | Lines | Row families |
| --- | --- | ---: | ---: | --- |
| Jupiter Swap | CH | 27 | 4,860 | 1 account row, 17 normal instruction rows, 6 event rows, 1 shared helper module, 2 module files |
| Jupiter Swap | PG | 21 | 4,921 | 1 account row, 17 normal instruction rows, 1 CPI-event row, 2 module files |
| Token Program | CH | 34 | 4,618 | 3 account rows, 28 instruction rows, 1 shared helper module, 2 module files |
| Token Program | PG | 33 | 7,037 | 3 account rows, 28 instruction rows, 2 module files |

</details>

<details>
<summary><strong>Example file counts</strong></summary>

| Side | File | Lines |
| --- | --- | ---: |
| CH | `examples/jupiter-swap-clickhouse/.env.example` | 22 |
| CH | `examples/jupiter-swap-clickhouse/Cargo.toml` | 21 |
| CH | `examples/jupiter-swap-clickhouse/README.md` | 242 |
| CH | `examples/jupiter-swap-clickhouse/src/main.rs` | 260 |
| CH | `examples/token-program-clickhouse/.env.example` | 15 |
| CH | `examples/token-program-clickhouse/Cargo.toml` | 20 |
| CH | `examples/token-program-clickhouse/README.md` | 92 |
| CH | `examples/token-program-clickhouse/src/main.rs` | 144 |
| PG | `examples/postgres-graphql/.env.example` | 11 |
| PG | `examples/postgres-graphql/Cargo.toml` | 25 |
| PG | `examples/postgres-graphql/compose.yaml` | 19 |
| PG | `examples/postgres-graphql/migrations/001_init.sql` | 8 |
| PG | `examples/postgres-graphql/README.md` | 93 |
| PG | `examples/postgres-graphql/src/main.rs` | 178 |

</details>

## Scoped Path Tree

Status legend:

- `[CH]` = ClickHouse sink path
- `[PG]` = Postgres sink path

```text
carbon/
├── crates/core/src/
│   ├── [CH] clickhouse/
│   │   ├── admin.rs
│   │   ├── config.rs
│   │   ├── http.rs
│   │   ├── metrics.rs
│   │   ├── mod.rs
│   │   ├── processors.rs
│   │   ├── rows/mod.rs
│   │   └── writer.rs
│   └── [PG] postgres/
│       ├── metadata.rs
│       ├── mod.rs
│       ├── operations.rs
│       ├── primitives.rs
│       ├── processors.rs
│       └── rows.rs
├── packages/renderer/
│   ├── [CH] src/clickhouseDdl.ts
│   ├── [CH] src/clickhouseRowMapper.ts
│   ├── [CH] templates/*ClickHouse*.njk
│   ├── [CH] templates/clickhouseRowPage.njk
│   ├── [CH] templates/clickhouseTypesPage.njk
│   ├── [CH] templates/eventInstructionClickHouseRowPage.njk
│   ├── [CH] test/clickhouse-renderer.test.cjs
│   ├── [PG] src/postgresRowMapper.ts
│   ├── [PG] src/getPostgresTypeManifestVisitor.ts
│   ├── [PG] src/utils/postgresHelpers.ts
│   └── [PG] templates/*Postgres*.njk / postgresRowPage.njk
├── decoders/
│   ├── jupiter-swap-decoder/src/
│   │   ├── [CH] accounts/clickhouse/
│   │   ├── [CH] instructions/clickhouse/
│   │   │   └── types.rs
│   │   ├── [PG] accounts/postgres/
│   │   └── [PG] instructions/postgres/
│   └── token-program-decoder/src/
│       ├── [CH] accounts/clickhouse/
│       ├── [CH] instructions/clickhouse/
│       │   └── types.rs
│       ├── [PG] accounts/postgres/
│       └── [PG] instructions/postgres/
└── examples/
    ├── [CH] jupiter-swap-clickhouse/
    ├── [CH] token-program-clickhouse/
    └── [PG] postgres-graphql/
```

## Technical Reading

- ClickHouse is not a direct line-for-line Postgres port. It is a separate landing-table sink with local batching, append-only identity, ClickHouse schema generation, managed schema reconciliation, and runtime reliability controls.
- Postgres remains simpler in core because the database client handles query execution through `sqlx`, rows are upserted immediately, and generic JSON tables exist for schema-less storage.
- The generated ClickHouse canaries are no longer the main source of line-size overhead. Shared helper extraction and core DDL/metadata builders make the committed Jupiter and Token Program ClickHouse outputs comparable to or smaller than their Postgres outputs.
- The remaining ClickHouse footprint is mostly in core/runtime and renderer support, where ClickHouse has capabilities that Postgres does not mirror: HTTP insert transport, per-buffer batching, async-wait insert settings, retry/backoff, dedup tokens, query IDs, schema drift validation/repair, production DDL modes, and structured ClickHouse type planning.
- Jupiter is intentionally different for CPI/events: ClickHouse currently keeps one typed landing table per generated event row family, while Postgres has one generated `cpi_event` row module.
- ClickHouse committed decoder output is canary-limited. Broad Postgres decoder coverage in the repo should not be interpreted as missing ClickHouse parity for every decoder until broad ClickHouse regeneration is intentionally performed.
