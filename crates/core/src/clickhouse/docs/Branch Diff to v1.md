# Branch Diff: `sevenlabs-hq/carbon:main` To `clickhouse-upstream-v1`

This document is a technical summary of the actual Git diff from `sevenlabs-hq/carbon`'s `main` branch to `clickhouse-upstream-v1`, filtered to implementation code. It is intentionally derived from Git commands and should be recomputed whenever either branch moves.

`main` in this worktree is our fork's mirror of `sevenlabs-hq/carbon:main`. At the time of this recomputation, `main` and `upstream/main` point to the same commit.

## Diff Basis

Commands used:

```bash
TZ=UTC date '+%Y-%m-%d %H:%M:%S %Z (%z)'
git rev-parse --abbrev-ref HEAD
git rev-parse main
git rev-parse upstream/main
git rev-parse HEAD
git merge-base main HEAD
git rev-list --left-right --count main...HEAD
git diff --name-status main...HEAD -- '*.rs' '*.ts' '*.njk' '*.cjs'
git diff --numstat main...HEAD -- '*.rs' '*.ts' '*.njk' '*.cjs'
git diff --stat main...HEAD -- '*.rs' '*.ts' '*.njk' '*.cjs'
git diff --shortstat main...HEAD -- '*.rs' '*.ts' '*.njk' '*.cjs'
```

Filtered scope:

- Included: Rust (`.rs`), TypeScript (`.ts`), renderer templates (`.njk`), and CommonJS renderer tests (`.cjs`).
- Excluded: docs and READMEs, CI, manifests, lockfiles, environment/config files, JSON dashboards, shell scripts, and other non-code artifacts.

Legend:

- `A`: added implementation file.
- `M`: modified implementation file.

The `A`/`M` inventory is recomputed from `git diff --name-status main...HEAD`. The current filtered inventory is `80` added implementation files and `36` modified implementation files; the consolidated summaries below follow that status split.

Computed values:

- Diff check timestamp: `2026-06-11 06:44:52 UTC (+0000)`
- Current branch: `clickhouse-upstream-v1`
- Reference branch tip: `sevenlabs-hq/carbon:main` at `f1797b485ed3232d2f95a7e0e5fb0316eafd770a`
- Local mirror tip: `main` at `f1797b485ed3232d2f95a7e0e5fb0316eafd770a`
- Compare branch tip: `clickhouse-upstream-v1` at `f5905dd4b84ab42cbef4e967e1133cb558911034`
- Merge base: `f1797b485ed3232d2f95a7e0e5fb0316eafd770a`
- Commit relation: `0` commits behind `sevenlabs-hq/carbon:main`, `63` commits ahead
- Total changed implementation files: `116`
- Added implementation files: `80`
- Modified implementation files: `36`
- Insertions: `13,786`
- Deletions: `175`
- Git shortstat: ` 116 files changed, 13786 insertions(+), 175 deletions(-)`

## Implementation Diff Footprint

This section excludes docs, README files, CI, manifests, lockfiles, config, JSON dashboards, and scripts. Line churn means added lines plus deleted lines.

<strong>Added vs modified implementation churn</strong>

<div style="display: flex; width: 100%; height: 24px; overflow: hidden;">
  <div title="Lines in added implementation files: 12,766 lines, 91.4%" style="width: 91.44%; background: #6B7280; color: #F8FAFC; line-height: 24px;">&nbsp;Added 91.4%</div>
  <div title="Changed-existing-code line churn: 1,195 lines, 8.6%" style="width: 8.56%; background: #374151; color: #F8FAFC; line-height: 24px;">&nbsp;Modified 8.6%</div>
</div>

<table width="100%">
<tr>
<td><font color="#6B7280">■</font> Added: <strong>12,766</strong> lines / <strong>91.4%</strong></td>
<td><font color="#374151">■</font> Modified: <strong>1,195</strong> lines / <strong>8.6%</strong></td>
<td align="right">Total: <strong>13,961</strong> lines</td>
</tr>
</table>

<strong>Section footprint</strong>

Bars show each section's share of total filtered implementation line churn. The darker patch inside a bar is changed-existing-code churn for that section.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;" cellspacing="0" cellpadding="6">
<colgroup>
<col style="width: 18%;" />
<col style="width: 66%;" />
<col style="width: 16%;" />
</colgroup>
<thead>
<tr>
<th align="left">Section</th>
<th align="left">Footprint</th>
<th align="right">A/M</th>
</tr>
</thead>
<tbody>
<tr>
<td>Decoders</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="Lines in added files: 5,104" style="width: 36.56%; background: #435A6F;">&nbsp;</div>
  <div title="Changed-existing-code line churn: 226" style="width: 1.62%; background: #2F4050;">&nbsp;</div>
  <div style="width: 61.82%;">&nbsp;</div>
</div>
<strong>38.2%</strong> / 5,330 lines
</td>
<td align="right" nowrap>5,104 / 226</td>
</tr>
<tr>
<td>CH Sink</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="Lines in added files: 4,643" style="width: 33.26%; background: #4F684E;">&nbsp;</div>
  <div style="width: 66.74%;">&nbsp;</div>
</div>
<strong>33.3%</strong> / 4,643 lines
</td>
<td align="right" nowrap>4,643 / 0</td>
</tr>
<tr>
<td>Renderer &amp; CLI</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="Lines in added files: 2,562" style="width: 18.35%; background: #66547D;">&nbsp;</div>
  <div title="Changed-existing-code line churn: 606" style="width: 4.34%; background: #493C5A;">&nbsp;</div>
  <div style="width: 77.31%;">&nbsp;</div>
</div>
<strong>22.7%</strong> / 3,168 lines
</td>
<td align="right" nowrap>2,562 / 606</td>
</tr>
<tr>
<td>Examples</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="Lines in added files: 457" style="width: 3.27%; background: #52645E;">&nbsp;</div>
  <div style="width: 96.73%;">&nbsp;</div>
</div>
<strong>3.3%</strong> / 457 lines
</td>
<td align="right" nowrap>457 / 0</td>
</tr>
<tr>
<td>Core Pipeline</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="Changed-existing-code line churn: 58" style="width: 0.42%; background: #4E3939;">&nbsp;</div>
  <div style="width: 99.58%;">&nbsp;</div>
</div>
<strong>0.4%</strong> / 58 lines
</td>
<td align="right" nowrap>0 / 58</td>
</tr>
<tr>
<td>Other</td>
<td>
<div style="display: flex; width: 100%; height: 18px; overflow: hidden;">
  <div title="Changed-existing-code line churn: 305" style="width: 2.18%; background: #554733;">&nbsp;</div>
  <div style="width: 97.82%;">&nbsp;</div>
</div>
<strong>2.2%</strong> / 305 lines
</td>
<td align="right" nowrap>0 / 305</td>
</tr>
</tbody>
</table>
</div>

<code>A/M</code> = lines in added files / modified-existing-code line churn.

<code>Other</code> contains datasource reliability, Prometheus metrics exporter, and version metadata changes.

## Consolidated File-By-File Summary

Line values are insertions/deletions for the filtered file diff. Modules are collapsed by default.

<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Core ClickHouse runtime</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>9 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>4643 / 0 lines</code></span></summary>

New ClickHouse sink runtime, row contracts, schema reconciliation, HTTP transport, buffering, retries, and metrics.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>1052/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>admin.rs</code></td>
<td>Implements ClickHouse admin execution, managed-table DDL helpers, schema/layout reconciliation, safe enum-extension repair, non-replicated MergeTree dedup defaults, and admin retry tests.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>376/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>config.rs</code></td>
<td>Defines ClickHouse endpoint/auth config, row context, batching/backpressure, transport, retry, sync default and async-wait insert settings, exact-batch dedup defaults, and database-URL parsing.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>248/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>http.rs</code></td>
<td>Implements ClickHouse HTTP client construction, auth, query/settings posting, gzip request bodies, response parsing, and retryable/permanent error classification.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>285/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>metrics.rs</code></td>
<td>Registers aggregate instruction/account ClickHouse metrics and helper recorders for rows, bytes, retries, backpressure, buffer state, flush batches, failures, and flush latency.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>26/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Re-exports ClickHouse admin, config, processor, metrics, row, and writer APIs from the feature-gated runtime module.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>406/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>processors.rs</code></td>
<td>Adds shared ClickHouse processor plumbing plus account and instruction processors that convert generated wrappers into rows, buffer them, record metrics, and drain on finalize.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>35/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>retry.rs</code></td>
<td>Provides retry eligibility checks and capped exponential backoff with optional jitter for ClickHouse HTTP/admin operations.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>663/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>rows/<wbr>mod.rs</code></td>
<td>Defines row/table contracts, instruction/event/account landing metadata, common column specs, deterministic IDs, enum helpers, and generated-row table/dispatch macros.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>1552/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>clickhouse/<wbr>writer.rs</code></td>
<td>Implements per-table/partition JSON buffering, row/byte thresholds, stale background flushing, global backpressure, retries, exact-batch dedup tokens, query IDs, snapshots, metrics, and shutdown drain.</td>
</tr>
</tbody>
</table>
</div>

</details>
<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Core pipeline lifecycle</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>8 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>58 / 0 lines</code></span></summary>

Core processor inputs, processor lifecycle, and pipeline finalization changes required by buffered sinks.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>5/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>account.rs</code></td>
<td>Forwards account pipe finalization into the wrapped processor so buffered account sinks can drain on shutdown.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>5/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>account_<wbr>deletion.rs</code></td>
<td>Forwards account-deletion pipe finalization into the wrapped processor for shutdown lifecycle consistency.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>5/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>block_<wbr>details.rs</code></td>
<td>Forwards block-details pipe finalization into the wrapped processor for shutdown lifecycle consistency.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>5/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>instruction.rs</code></td>
<td>Forwards instruction pipe finalization into the wrapped processor so buffered instruction/event sinks can drain on shutdown.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>2/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>lib.rs</code></td>
<td>Exports the optional ClickHouse runtime module behind the `clickhouse` feature.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>27/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>pipeline.rs</code></td>
<td>Finalizes all pipe families before exporter shutdown on cancellation, immediate ctrl-C shutdown, and closed-receiver shutdown paths.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>4/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>processor.rs</code></td>
<td>Adds the default no-op `Processor::finalize()` lifecycle hook used by buffered sinks.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>5/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">crates/<wbr>core/<wbr>src/<wbr>transaction.rs</code></td>
<td>Forwards transaction pipe finalization into the wrapped processor while preserving flat decoded-instruction routing.</td>
</tr>
</tbody>
</table>
</div>

</details>
<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Jupiter Swap decoder canary</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>32 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>3005 / 1 lines</code></span></summary>

Generated Jupiter Swap ClickHouse account, instruction, CPI/event rows, and decoder wiring.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>117/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Defines the Jupiter TokenLedger account ClickHouse processor alias, account table options, managed schema migration, config defaults, and setup helpers.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>58/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Persists Jupiter TokenLedger account snapshots with token account and amount payload columns plus common account landing metadata.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>3/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>accounts/<wbr>mod.rs</code></td>
<td>Feature-gates the account ClickHouse module and decodes Jupiter-owned TokenLedger accounts into the account enum.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>58/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>best_<wbr>swap_<wbr>out_<wbr>amount_<wbr>violation_<wbr>event_<wbr>row.rs</code></td>
<td>Persists best-swap threshold violation events with expected and actual output amount payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>60/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>candidate_<wbr>swap_<wbr>quote_<wbr>error_<wbr>event_<wbr>row.rs</code></td>
<td>Persists candidate quote failure events with candidate index, input amount, and error code payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>60/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>candidate_<wbr>swap_<wbr>results_<wbr>event_<wbr>row.rs</code></td>
<td>Persists candidate swap result events as structured arrays of generated ClickHouse candidate-result helper types.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>53/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>claim_<wbr>row.rs</code></td>
<td>Persists Jupiter `claim` instruction rows with the claim id payload column and common instruction landing metadata.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>53/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>claim_<wbr>token_<wbr>row.rs</code></td>
<td>Persists Jupiter `claim_token` instruction rows with the claim id payload column and common instruction landing metadata.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>57/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>close_<wbr>token_<wbr>row.rs</code></td>
<td>Persists Jupiter `close_token` instruction rows with id and burn-all payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>51/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>close_<wbr>wsol_<wbr>token_<wbr>account_<wbr>row.rs</code></td>
<td>Persists Jupiter WSOL cleanup instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>54/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>create_<wbr>token_<wbr>account_<wbr>row.rs</code></td>
<td>Persists Jupiter token-account creation instruction rows with the derived bump payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>51/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>create_<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Persists Jupiter TokenLedger creation instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>71/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>exact_<wbr>out_<wbr>route_<wbr>row.rs</code></td>
<td>Persists exact-out route instructions with structured route-plan steps plus output amount, quoted input, slippage, and platform-fee payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>75/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>exact_<wbr>out_<wbr>route_<wbr>v2_<wbr>row.rs</code></td>
<td>Persists v2 exact-out route instructions with structured v2 route-plan steps, amount fields, platform fee, and positive-slippage payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>60/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>fee_<wbr>event_<wbr>event_<wbr>row.rs</code></td>
<td>Persists Jupiter fee events with fee account, mint, and amount payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>233/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Defines explicit instruction/event row exports, table options, row enum dispatch, CPI-event routing, managed schema migration, config defaults, and setup helpers.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>71/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>route_<wbr>row.rs</code></td>
<td>Persists standard route instructions with structured route-plan steps plus input amount, quoted output, slippage, and platform-fee payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>74/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>route_<wbr>v2_<wbr>row.rs</code></td>
<td>Persists v2 route instructions with structured v2 route-plan steps, amount fields, platform fee, and positive-slippage payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>69/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>route_<wbr>with_<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Persists token-ledger route instructions with structured route-plan steps, quoted output, slippage, and platform-fee payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>51/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>set_<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Persists `set_token_ledger` instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>76/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>exact_<wbr>out_<wbr>route_<wbr>row.rs</code></td>
<td>Persists shared-account exact-out route instructions with route id, structured route-plan steps, amount, slippage, and platform-fee payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>81/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>exact_<wbr>out_<wbr>route_<wbr>v2_<wbr>row.rs</code></td>
<td>Persists shared-account v2 exact-out route instructions with route id, structured v2 route-plan steps, amount fields, platform fee, and positive slippage.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>75/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>route_<wbr>row.rs</code></td>
<td>Persists shared-account route instructions with route id, structured route-plan steps, input/output amounts, slippage, and platform-fee payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>78/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>route_<wbr>v2_<wbr>row.rs</code></td>
<td>Persists shared-account v2 route instructions with route id, structured v2 route-plan steps, amount fields, platform fee, and positive slippage.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>75/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>shared_<wbr>accounts_<wbr>route_<wbr>with_<wbr>token_<wbr>ledger_<wbr>row.rs</code></td>
<td>Persists shared-account token-ledger route instructions with route id, structured route-plan steps, quoted output, slippage, and platform fee.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>66/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>swap_<wbr>event_<wbr>event_<wbr>row.rs</code></td>
<td>Persists single-swap execution events with AMM, input/output mints, and input/output amount payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>60/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>swaps_<wbr>event_<wbr>event_<wbr>row.rs</code></td>
<td>Persists grouped swap execution events as structured arrays of generated v2 swap-event helper types.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>1074/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>types.rs</code></td>
<td>Defines generated ClickHouse helper types, enum/tuple serializers, and shared DDL constants for structured payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>19/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>cpi_<wbr>event.rs</code></td>
<td>Adds CPI/event account metadata construction so decoded Anchor event CPIs can retain program, event-authority, and remaining-account context.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>14/1</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>instructions/<wbr>mod.rs</code></td>
<td>Feature-gates the ClickHouse instruction module and decodes Anchor event-CPI payloads before ordinary Jupiter instruction matching.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>1/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>types/<wbr>candidate_<wbr>swap.rs</code></td>
<td>Adds the `ZeroFiSwapV2` candidate-swap variant used by regenerated Jupiter route-plan ClickHouse helper types.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>7/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>jupiter-swap-decoder/<wbr>src/<wbr>types/<wbr>swap.rs</code></td>
<td>Adds regenerated Jupiter swap variants including `PumpWrappedBuyV5`, `PumpWrappedSellV5`, and `ZeroFiSwapV2` for route-plan row mapping.</td>
</tr>
</tbody>
</table>
</div>

</details>
<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Token Program decoder canary</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>39 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>2261 / 63 lines</code></span></summary>

Generated Token Program ClickHouse account/instruction rows plus SPL Token account decoding corrections.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>73/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>mint_<wbr>row.rs</code></td>
<td>Persists SPL Token mint account snapshots with mint/freeze authorities, supply, decimals, and initialization payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>125/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Defines explicit Token Program account row exports, enum dispatch, MergeTree options, managed schema migration, config defaults, and setup helpers.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>68/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>multisig_<wbr>row.rs</code></td>
<td>Persists SPL Token multisig account snapshots with signer threshold, signer count, initialization state, and signer pubkey payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>82/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>accounts/<wbr>clickhouse/<wbr>token_<wbr>row.rs</code></td>
<td>Persists SPL Token account snapshots with mint, token owner, balance, delegate, state, native reserve, delegated amount, and close authority columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>8/10</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>accounts/<wbr>mint.rs</code></td>
<td>Replaces Borsh byte decoding with conversion from official SPL Token mint state so generated mint rows use the correct Pack layout.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>79/33</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>accounts/<wbr>mod.rs</code></td>
<td>Adds the ClickHouse account module and decodes Token Program Mint, Token, and Multisig accounts through official SPL Token Pack unpacking, with a token-account layout regression test.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>7/10</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>accounts/<wbr>multisig.rs</code></td>
<td>Replaces Borsh byte decoding with conversion from official SPL Token multisig state so signer fields use the correct Pack layout.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>21/10</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>accounts/<wbr>token.rs</code></td>
<td>Replaces Borsh byte decoding with conversion from official SPL Token account state, including COption, owner, delegate, native reserve, and account-state mapping.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>54/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>amount_<wbr>to_<wbr>ui_<wbr>amount_<wbr>row.rs</code></td>
<td>Persists `amount_to_ui_amount` instruction rows with the raw token amount payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>58/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>approve_<wbr>checked_<wbr>row.rs</code></td>
<td>Persists checked approval instruction rows with amount and decimals payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>53/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>approve_<wbr>row.rs</code></td>
<td>Persists approval instruction rows with the delegated amount payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>62/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>batch_<wbr>row.rs</code></td>
<td>Persists batch instruction rows with structured batch-item payload data.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>57/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>burn_<wbr>checked_<wbr>row.rs</code></td>
<td>Persists checked burn instruction rows with amount and decimals payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>53/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>burn_<wbr>row.rs</code></td>
<td>Persists burn instruction rows with the burned amount payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>50/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>close_<wbr>account_<wbr>row.rs</code></td>
<td>Persists close-account instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>50/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>freeze_<wbr>account_<wbr>row.rs</code></td>
<td>Persists freeze-account instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>51/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>get_<wbr>account_<wbr>data_<wbr>size_<wbr>row.rs</code></td>
<td>Persists get-account-data-size instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>54/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>account2_<wbr>row.rs</code></td>
<td>Persists `initialize_account2` instruction rows with the owner payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>54/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>account3_<wbr>row.rs</code></td>
<td>Persists `initialize_account3` instruction rows with the owner payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>51/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>account_<wbr>row.rs</code></td>
<td>Persists initialize-account instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>51/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>immutable_<wbr>owner_<wbr>row.rs</code></td>
<td>Persists initialize-immutable-owner instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>64/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>mint2_<wbr>row.rs</code></td>
<td>Persists `initialize_mint2` instruction rows with decimals, mint authority, and freeze authority payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>64/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>mint_<wbr>row.rs</code></td>
<td>Persists initialize-mint instruction rows with decimals, mint authority, and freeze authority payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>54/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>multisig2_<wbr>row.rs</code></td>
<td>Persists `initialize_multisig2` instruction rows with the required signer count payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>54/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>initialize_<wbr>multisig_<wbr>row.rs</code></td>
<td>Persists initialize-multisig instruction rows with the required signer count payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>58/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>mint_<wbr>to_<wbr>checked_<wbr>row.rs</code></td>
<td>Persists checked mint-to instruction rows with amount and decimals payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>53/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>mint_<wbr>to_<wbr>row.rs</code></td>
<td>Persists mint-to instruction rows with the minted amount payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>243/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>mod.rs</code></td>
<td>Defines explicit Token Program instruction row exports, enum dispatch, MergeTree options, managed schema migration, config defaults, and setup helpers.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>50/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>revoke_<wbr>row.rs</code></td>
<td>Persists revoke instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>60/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>set_<wbr>authority_<wbr>row.rs</code></td>
<td>Persists set-authority instruction rows with authority type and optional new-authority payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>50/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>sync_<wbr>native_<wbr>row.rs</code></td>
<td>Persists sync-native instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>50/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>thaw_<wbr>account_<wbr>row.rs</code></td>
<td>Persists thaw-account instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>58/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>transfer_<wbr>checked_<wbr>row.rs</code></td>
<td>Persists checked transfer instruction rows with amount and decimals payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>53/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>transfer_<wbr>row.rs</code></td>
<td>Persists transfer instruction rows with the transferred amount payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>27/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>types.rs</code></td>
<td>Defines the generated ClickHouse helper type for structured Token Program batch instruction items.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>54/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>ui_<wbr>amount_<wbr>to_<wbr>amount_<wbr>row.rs</code></td>
<td>Persists `ui_amount_to_amount` instruction rows with the UI amount payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>54/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>unwrap_<wbr>lamports_<wbr>row.rs</code></td>
<td>Persists unwrap-lamports instruction rows with the lamport amount payload column.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>51/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>clickhouse/<wbr>withdraw_<wbr>excess_<wbr>lamports_<wbr>row.rs</code></td>
<td>Persists withdraw-excess-lamports instruction rows with common instruction landing metadata and no extra payload columns.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>3/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">decoders/<wbr>token-program-decoder/<wbr>src/<wbr>instructions/<wbr>mod.rs</code></td>
<td>Feature-gates the Token Program instruction ClickHouse module in the generated instruction decoder family.</td>
</tr>
</tbody>
</table>
</div>

</details>
<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Renderer</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>18 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>3047 / 20 lines</code></span></summary>

Codama renderer support for ClickHouse typed rows, structured mapping, DDL modes, schema metadata, and tests.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>43/3</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>src/<wbr>cargoTomlGenerator.ts</code></td>
<td>Adds generated decoder ClickHouse feature wiring, `chrono` dependency support, and classic SPL Token Program Pack dependencies.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>232/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>src/<wbr>clickhouseDdl.ts</code></td>
<td>Builds renderer-controlled MergeTree, ReplicatedMergeTree, and Distributed DDL options, defaults, codecs, and deduplication settings.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>1174/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>src/<wbr>clickhouseRowMapper.ts</code></td>
<td>Plans structured ClickHouse row fields/types/conversions from decoder schema, including tuples, arrays, enums, payload unions, and unsupported-type failures.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>226/5</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>src/<wbr>getRenderMapVisitor.ts</code></td>
<td>Routes ClickHouse render options into generated account, instruction, event, and type pages, including helper de-duplication, DDL contexts, and token-program Pack imports.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>4/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>src/<wbr>index.ts</code></td>
<td>Exports ClickHouse DDL helpers, row mapper, render options, and row-plan types from the renderer package API.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>13/1</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>src/<wbr>utils/<wbr>helpers.ts</code></td>
<td>Normalizes program, original IDL, and package names for Token and Token-2022 program detection.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>136/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>accountsClickHouseMod.njk</code></td>
<td>Generates account ClickHouse modules with explicit row exports, account processor aliases, table options, migration runner, config defaults, and setup helpers.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>94/3</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>accountsMod.njk</code></td>
<td>Adds feature-gated account ClickHouse modules and emits official SPL Token Pack-based account decoding/tests for classic Token Program accounts.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>52/1</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>accountsPage.njk</code></td>
<td>Emits classic SPL Token `From` conversions for Mint, Token, and Multisig account structs alongside the existing Token-2022 conversion path.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>114/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>clickhouseRowPage.njk</code></td>
<td>Generates typed account/instruction ClickHouse row structs with landing metadata, payload column specs, source-to-row conversion, and row macros.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>11/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>clickhouseTypesPage.njk</code></td>
<td>Emits shared ClickHouse helper definitions and enum helper imports for generated structured payload types.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>67/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>eventInstructionClickHouseRowPage.njk</code></td>
<td>Generates event ClickHouse row structs with event landing metadata, payload column specs, source-to-row conversion, and event row macros.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>17/1</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>eventInstructionPage.njk</code></td>
<td>Adds `from_instruction_accounts` so generated CPI event instructions preserve program, event-authority, and remaining account metadata.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>194/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>instructionsClickHouseMod.njk</code></td>
<td>Generates instruction/event ClickHouse modules with explicit row exports, dispatch, table options, migration runner, config defaults, and setup helpers.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>20/4</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>instructionsMod.njk</code></td>
<td>Adds feature-gated instruction ClickHouse modules and decodes CPI events before ordinary instruction matching in generated instruction decoders.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>1/1</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>lib.njk</code></td>
<td>Normalizes the generated `lib.rs` template trailing newline without changing generated decoder behavior.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>15/1</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>templates/<wbr>macros.njk</code></td>
<td>Adds the `clickHouseColumnSpec` template macro for generated payload column specs, including custom DDL types and codecs.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>634/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>renderer/<wbr>test/<wbr>clickhouse-renderer.test.cjs</code></td>
<td>Tests ClickHouse module templates, DDL modes/options/codecs, structured row mapping, Token Program Pack generation, reserved-column collisions, strict JSON fallback behavior, and generated canaries.</td>
</tr>
</tbody>
</table>
</div>

</details>
<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>CLI</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>5 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>95 / 6 lines</code></span></summary>

CLI plumbing for ClickHouse generation and renderer option parity.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>70/4</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>cli/<wbr>src/<wbr>cli.ts</code></td>
<td>Adds `--with-clickhouse` and `--clickhouse-options` parsing, accepts JSON or file-based options, resolves enabled state, and passes ClickHouse options through parse/scaffold commands.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>10/1</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>cli/<wbr>src/<wbr>lib/<wbr>cargoTomlGenerator.ts</code></td>
<td>Adds ClickHouse feature propagation to scaffolded decoder and `carbon-core` dependencies plus the scaffolded indexer feature list.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>10/1</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>cli/<wbr>src/<wbr>lib/<wbr>decoder.ts</code></td>
<td>Threads `withClickHouse` render options through Codama and Anchor decoder generation paths.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>2/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>cli/<wbr>src/<wbr>lib/<wbr>prompts.ts</code></td>
<td>Carries the optional `withClickHouse` scaffold setting through interactive scaffold options.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>3/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>cli/<wbr>src/<wbr>lib/<wbr>scaffold.ts</code></td>
<td>Adds `withClickHouse` to scaffold options, generated template context, and generated README feature output.</td>
</tr>
</tbody>
</table>
</div>

</details>
<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Examples</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>2 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>457 / 0 lines</code></span></summary>

Thin real-world ClickHouse canaries for Jupiter Swap and Token Program data ingestion.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>261/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">examples/<wbr>jupiter-swap-clickhouse/<wbr>src/<wbr>main.rs</code></td>
<td>Runs the Jupiter Swap ClickHouse canary with finalized block crawling, instruction/event rows, metrics, async insert opt-in, and live-mode TokenLedger account snapshots.</td>
</tr><tr>
<td align="left" style="white-space: nowrap;"><code>196/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">examples/<wbr>token-program-clickhouse/<wbr>src/<wbr>main.rs</code></td>
<td>Runs the fixed-USDC Token Program ClickHouse canary: snapshots the hardwired USDC mint, authority multisigs, and holding account with finalized `getMultipleAccounts`, then tails finalized blocks for instruction rows.</td>
</tr>
</tbody>
</table>
</div>

</details>
<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Datasource reliability</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>1 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>197 / 84 lines</code></span></summary>

RPC block crawler drain, backpressure, retry, cancellation, and skip/error classification changes.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>197/84</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">datasources/<wbr>rpc-block-crawler-datasource/<wbr>src/<wbr>lib.rs</code></td>
<td>Makes the RPC block crawler drain queued blocks after the fetcher exits, await transaction sends for downstream backpressure, retry transient block-fetch errors, and skip permanent Solana block errors.</td>
</tr>
</tbody>
</table>
</div>

</details>
<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Metrics</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>1 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>22 / 1 lines</code></span></summary>

Prometheus metrics export behavior used by local ClickHouse sink monitoring.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>22/1</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">metrics/<wbr>prometheus-metrics/<wbr>src/<wbr>lib.rs</code></td>
<td>Sanitizes Carbon metric names for Prometheus-compatible output.</td>
</tr>
</tbody>
</table>
</div>

</details>
<details>
<summary style="white-space: nowrap;"><span style="display: inline-block; width: 45%;"><strong>Versions package</strong></span><span style="display: inline-block; width: 19%; text-align: left;"><code>1 files</code></span><span style="display: inline-block; width: 36%; text-align: right;"><code>1 / 0 lines</code></span></summary>

Version export update needed by generated package plumbing.

<div style="width: 100%; overflow-x: auto;">
<table style="width: 100%; table-layout: fixed;">
<colgroup>
<col style="width: 9%;" />
<col style="width: 30%;" />
<col style="width: 61%;" />
</colgroup>
<thead>
<tr>
<th align="left">Lines</th>
<th align="left">File</th>
<th align="left">Summary</th>
</tr>
</thead>
<tbody>
<tr>
<td align="left" style="white-space: nowrap;"><code>1/0</code></td>
<td style="white-space: normal; overflow-wrap: anywhere; word-break: break-word;"><code style="white-space: normal; overflow-wrap: anywhere; word-break: break-all;">packages/<wbr>versions/<wbr>src/<wbr>index.ts</code></td>
<td>Adds the `chrono` crate version used by generated ClickHouse row timestamp dependencies.</td>
</tr>
</tbody>
</table>
</div>

</details>

## Touched Path Tree

Status legend:

- `[A]` / `A` = added implementation file
- `[M]` / `M` = modified existing implementation file

<style>
.touched-path-tree-tabs input[type="radio"] { height: 1px; opacity: 0; position: absolute; width: 1px; }
.touched-path-tree-tabs .tree-mode-controls { margin: 0.85em 0 0; }
.touched-path-tree-tabs .tree-mode-controls + .tree-mode-controls { margin-top: 0.2em; margin-bottom: 1.1em; }
.touched-path-tree-tabs .tree-mode-controls label { color: #0969da; cursor: pointer; margin-left: 0; text-decoration: underline; text-decoration-thickness: 1px; text-underline-offset: 0.18em; }
.touched-path-tree-tabs .tree-mode-controls span { color: #64748b; margin-left: 0.45em; margin-right: 0.45em; }
.touched-path-tree-tabs .tree-view { display: none; }
.touched-path-tree-tabs .tree-code-window { border: 1px solid rgba(148, 163, 184, 0.24); border-radius: 12px; font-family: var(--vscode-editor-font-family, ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace); font-size: 0.92em; line-height: 1.18; margin: 0; overflow-x: auto; padding: 1em; }
.touched-path-tree-tabs .tree-code-window details, .touched-path-tree-tabs .tree-code-window summary, .touched-path-tree-tabs .tree-code-window .tree-file, .touched-path-tree-tabs .tree-code-window .tree-children { margin: 0 !important; padding: 0 !important; }
.touched-path-tree-tabs .tree-code-window summary, .touched-path-tree-tabs .tree-code-window .tree-file { display: block; line-height: 1.18; white-space: pre; }
.touched-path-tree-tabs .tree-code-window summary { cursor: pointer; list-style: none; list-style-type: none; }
.touched-path-tree-tabs .tree-code-window summary::marker { content: ""; font-size: 0; }
.touched-path-tree-tabs .tree-code-window summary::-webkit-details-marker { display: none; }
.touched-path-tree-tabs .tree-code-window .tree-status { color: #94a3b8; }
#tree-mode-first-level:checked ~ .tree-mode-controls label[for="tree-mode-first-level"] { color: #8250df; font-weight: 400; text-decoration-thickness: 1px; }
#tree-mode-second-level:checked ~ .tree-mode-controls label[for="tree-mode-second-level"] { color: #8250df; font-weight: 400; text-decoration-thickness: 1px; }
#tree-mode-expanded:checked ~ .tree-mode-controls label[for="tree-mode-expanded"] { color: #8250df; font-weight: 400; text-decoration-thickness: 1px; }
#tree-mode-collapsed:checked ~ .tree-mode-controls label[for="tree-mode-collapsed"] { color: #8250df; font-weight: 400; text-decoration-thickness: 1px; }
#tree-mode-carbon-core:checked ~ .tree-mode-controls label[for="tree-mode-carbon-core"] { color: #8250df; font-weight: 400; text-decoration-thickness: 1px; }
#tree-mode-ch-sink:checked ~ .tree-mode-controls label[for="tree-mode-ch-sink"] { color: #8250df; font-weight: 400; text-decoration-thickness: 1px; }
#tree-mode-decoders:checked ~ .tree-mode-controls label[for="tree-mode-decoders"] { color: #8250df; font-weight: 400; text-decoration-thickness: 1px; }
#tree-mode-cli-renderer:checked ~ .tree-mode-controls label[for="tree-mode-cli-renderer"] { color: #8250df; font-weight: 400; text-decoration-thickness: 1px; }
#tree-mode-first-level:checked ~ .tree-view-first-level { display: block; }
#tree-mode-second-level:checked ~ .tree-view-second-level { display: block; }
#tree-mode-expanded:checked ~ .tree-view-expanded { display: block; }
#tree-mode-collapsed:checked ~ .tree-view-collapsed { display: block; }
#tree-mode-carbon-core:checked ~ .tree-view-carbon-core { display: block; }
#tree-mode-ch-sink:checked ~ .tree-view-ch-sink { display: block; }
#tree-mode-decoders:checked ~ .tree-view-decoders { display: block; }
#tree-mode-cli-renderer:checked ~ .tree-view-cli-renderer { display: block; }
</style>

<div class="touched-path-tree-tabs">
<input id="tree-mode-first-level" type="radio" name="touched-path-tree-mode" checked>
<input id="tree-mode-second-level" type="radio" name="touched-path-tree-mode">
<input id="tree-mode-expanded" type="radio" name="touched-path-tree-mode">
<input id="tree-mode-collapsed" type="radio" name="touched-path-tree-mode">
<input id="tree-mode-carbon-core" type="radio" name="touched-path-tree-mode">
<input id="tree-mode-ch-sink" type="radio" name="touched-path-tree-mode">
<input id="tree-mode-decoders" type="radio" name="touched-path-tree-mode">
<input id="tree-mode-cli-renderer" type="radio" name="touched-path-tree-mode">

<nav class="tree-mode-controls" aria-label="Touched path tree controls">
<label for="tree-mode-expanded">expand all</label><span>/</span><label for="tree-mode-collapsed">collapse all</label><span>/</span><label for="tree-mode-first-level">1st level</label><span>/</span><label for="tree-mode-second-level">2nd level</label>
</nav>
<nav class="tree-mode-controls" aria-label="Touched path tree section controls">
<label for="tree-mode-carbon-core">Carbon Core</label><span>/</span><label for="tree-mode-ch-sink">CH Sink</label><span>/</span><label for="tree-mode-decoders">Decoders</label><span>/</span><label for="tree-mode-cli-renderer">CLI &amp; Renderer</label>
</nav>

<div class="tree-view tree-view-first-level">
<div class="tree-code-window">
<details class="tree-node" open><summary><span class="tree-prefix"></span><span class="tree-label">carbon/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">crates/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">core/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">rows/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> mod.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> admin.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> config.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> http.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> metrics.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> processors.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> retry.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[A]</span> writer.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account_deletion.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> block_details.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> instruction.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> lib.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> pipeline.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> processor.rs</div>
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> transaction.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">datasources/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">rpc-block-crawler-datasource/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">decoders/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> token_ledger_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> best_swap_out_amount_violation_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_quote_error_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_results_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_wsol_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> fee_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> set_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swap_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swaps_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> types.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   ├── </span><span class="tree-status">[M]</span> cpi_event.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       └── </span><span class="tree-label">types/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │           ├── </span><span class="tree-status">[M]</span> candidate_swap.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │           └── </span><span class="tree-status">[M]</span> swap.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> token_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mint.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> multisig.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[M]</span> token.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│           └── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│               ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> amount_to_ui_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> batch_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> close_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> freeze_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> get_account_data_size_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account3_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_immutable_owner_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> revoke_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> set_authority_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> sync_native_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> thaw_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> types.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> ui_amount_to_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> unwrap_lamports_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   └── </span><span class="tree-status">[A]</span> withdraw_excess_lamports_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│               └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">examples/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">prometheus-metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">└── </span><span class="tree-label">packages/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">cli/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │       ├── </span><span class="tree-label">lib/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> decoder.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> prompts.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   └── </span><span class="tree-status">[M]</span> scaffold.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[M]</span> cli.ts</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">renderer/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   │   ├── </span><span class="tree-label">utils/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   │   └── </span><span class="tree-status">[M]</span> helpers.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseDdl.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowMapper.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> getRenderMapVisitor.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">templates/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> accountsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseTypesPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> eventInstructionClickHouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> eventInstructionPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> instructionsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> instructionsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> lib.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> macros.njk</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">test/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[A]</span> clickhouse-renderer.test.cjs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    └── </span><span class="tree-label">versions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">        └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">            └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
</div></details>
</div></details>
</div></details>
</div>
</div>

<div class="tree-view tree-view-second-level">
<div class="tree-code-window">
<details class="tree-node" open><summary><span class="tree-prefix"></span><span class="tree-label">carbon/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">crates/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">core/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">rows/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> mod.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> admin.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> config.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> http.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> metrics.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> processors.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> retry.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[A]</span> writer.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account_deletion.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> block_details.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> instruction.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> lib.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> pipeline.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> processor.rs</div>
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> transaction.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">datasources/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">rpc-block-crawler-datasource/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">decoders/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> token_ledger_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> best_swap_out_amount_violation_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_quote_error_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_results_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_wsol_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> fee_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> set_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swap_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swaps_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> types.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   ├── </span><span class="tree-status">[M]</span> cpi_event.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       └── </span><span class="tree-label">types/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │           ├── </span><span class="tree-status">[M]</span> candidate_swap.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │           └── </span><span class="tree-status">[M]</span> swap.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> token_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mint.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> multisig.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[M]</span> token.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│           └── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│               ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> amount_to_ui_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> batch_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> close_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> freeze_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> get_account_data_size_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account3_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_immutable_owner_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> revoke_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> set_authority_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> sync_native_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> thaw_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> types.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> ui_amount_to_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> unwrap_lamports_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   └── </span><span class="tree-status">[A]</span> withdraw_excess_lamports_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│               └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">examples/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">prometheus-metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">└── </span><span class="tree-label">packages/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">cli/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │       ├── </span><span class="tree-label">lib/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> decoder.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> prompts.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   └── </span><span class="tree-status">[M]</span> scaffold.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[M]</span> cli.ts</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">renderer/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   │   ├── </span><span class="tree-label">utils/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   │   └── </span><span class="tree-status">[M]</span> helpers.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseDdl.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowMapper.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> getRenderMapVisitor.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">templates/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> accountsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseTypesPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> eventInstructionClickHouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> eventInstructionPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> instructionsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> instructionsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> lib.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> macros.njk</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">test/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[A]</span> clickhouse-renderer.test.cjs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    └── </span><span class="tree-label">versions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">        └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">            └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
</div></details>
</div></details>
</div></details>
</div>
</div>

<div class="tree-view tree-view-expanded">
<div class="tree-code-window">
<details class="tree-node" open><summary><span class="tree-prefix"></span><span class="tree-label">carbon/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">crates/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   └── </span><span class="tree-label">core/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">rows/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> mod.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> admin.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> config.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> http.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> metrics.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> processors.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> retry.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[A]</span> writer.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account_deletion.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> block_details.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> instruction.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> lib.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> pipeline.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> processor.rs</div>
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> transaction.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">datasources/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   └── </span><span class="tree-label">rpc-block-crawler-datasource/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">decoders/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> token_ledger_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> best_swap_out_amount_violation_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_quote_error_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_results_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_wsol_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> fee_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> set_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swap_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swaps_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> types.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   ├── </span><span class="tree-status">[M]</span> cpi_event.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">│   │       └── </span><span class="tree-label">types/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │           ├── </span><span class="tree-status">[M]</span> candidate_swap.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │           └── </span><span class="tree-status">[M]</span> swap.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> token_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mint.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> multisig.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[M]</span> token.rs</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">│           └── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│               ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> amount_to_ui_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> batch_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> close_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> freeze_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> get_account_data_size_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account3_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_immutable_owner_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> revoke_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> set_authority_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> sync_native_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> thaw_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> types.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> ui_amount_to_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> unwrap_lamports_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   └── </span><span class="tree-status">[A]</span> withdraw_excess_lamports_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│               └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">examples/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">metrics/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   └── </span><span class="tree-label">prometheus-metrics/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">└── </span><span class="tree-label">packages/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    ├── </span><span class="tree-label">cli/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    │       ├── </span><span class="tree-label">lib/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> decoder.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> prompts.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   └── </span><span class="tree-status">[M]</span> scaffold.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[M]</span> cli.ts</div>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">    ├── </span><span class="tree-label">renderer/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    │   │   ├── </span><span class="tree-label">utils/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   │   └── </span><span class="tree-status">[M]</span> helpers.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseDdl.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowMapper.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> getRenderMapVisitor.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">templates/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> accountsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseTypesPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> eventInstructionClickHouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> eventInstructionPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> instructionsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> instructionsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> lib.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> macros.njk</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">test/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[A]</span> clickhouse-renderer.test.cjs</div>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">    └── </span><span class="tree-label">versions/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">        └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">            └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
</div></details>
</div></details>
</div></details>
</div>
</div>

<div class="tree-view tree-view-collapsed">
<div class="tree-code-window">
<details class="tree-node" open><summary><span class="tree-prefix"></span><span class="tree-label">carbon/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">crates/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">core/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">rows/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> mod.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> admin.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> config.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> http.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> metrics.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> processors.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> retry.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[A]</span> writer.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account_deletion.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> block_details.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> instruction.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> lib.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> pipeline.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> processor.rs</div>
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> transaction.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">datasources/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">rpc-block-crawler-datasource/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">decoders/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> token_ledger_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> best_swap_out_amount_violation_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_quote_error_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_results_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_wsol_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> fee_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> set_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swap_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swaps_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> types.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   ├── </span><span class="tree-status">[M]</span> cpi_event.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       └── </span><span class="tree-label">types/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │           ├── </span><span class="tree-status">[M]</span> candidate_swap.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │           └── </span><span class="tree-status">[M]</span> swap.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> token_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mint.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> multisig.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[M]</span> token.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│           └── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│               ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> amount_to_ui_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> batch_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> close_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> freeze_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> get_account_data_size_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account3_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_immutable_owner_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> revoke_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> set_authority_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> sync_native_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> thaw_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> types.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> ui_amount_to_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> unwrap_lamports_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   └── </span><span class="tree-status">[A]</span> withdraw_excess_lamports_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│               └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">examples/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">prometheus-metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">└── </span><span class="tree-label">packages/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">cli/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │       ├── </span><span class="tree-label">lib/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> decoder.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> prompts.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   └── </span><span class="tree-status">[M]</span> scaffold.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[M]</span> cli.ts</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">renderer/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   │   ├── </span><span class="tree-label">utils/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   │   └── </span><span class="tree-status">[M]</span> helpers.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseDdl.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowMapper.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> getRenderMapVisitor.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">templates/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> accountsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseTypesPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> eventInstructionClickHouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> eventInstructionPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> instructionsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> instructionsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> lib.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> macros.njk</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">test/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[A]</span> clickhouse-renderer.test.cjs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    └── </span><span class="tree-label">versions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">        └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">            └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
</div></details>
</div></details>
</div></details>
</div>
</div>

<div class="tree-view tree-view-carbon-core">
<div class="tree-code-window">
<details class="tree-node" open><summary><span class="tree-prefix"></span><span class="tree-label">carbon/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">crates/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   └── </span><span class="tree-label">core/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">rows/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> mod.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> admin.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> config.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> http.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> metrics.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> processors.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> retry.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[A]</span> writer.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account_deletion.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> block_details.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> instruction.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> lib.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> pipeline.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> processor.rs</div>
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> transaction.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">datasources/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">rpc-block-crawler-datasource/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">decoders/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> token_ledger_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> best_swap_out_amount_violation_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_quote_error_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_results_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_wsol_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> fee_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> set_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swap_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swaps_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> types.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   ├── </span><span class="tree-status">[M]</span> cpi_event.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       └── </span><span class="tree-label">types/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │           ├── </span><span class="tree-status">[M]</span> candidate_swap.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │           └── </span><span class="tree-status">[M]</span> swap.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> token_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mint.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> multisig.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[M]</span> token.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│           └── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│               ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> amount_to_ui_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> batch_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> close_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> freeze_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> get_account_data_size_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account3_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_immutable_owner_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> revoke_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> set_authority_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> sync_native_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> thaw_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> types.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> ui_amount_to_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> unwrap_lamports_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   └── </span><span class="tree-status">[A]</span> withdraw_excess_lamports_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│               └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">examples/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">prometheus-metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">└── </span><span class="tree-label">packages/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">cli/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │       ├── </span><span class="tree-label">lib/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> decoder.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> prompts.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   └── </span><span class="tree-status">[M]</span> scaffold.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[M]</span> cli.ts</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">renderer/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   │   ├── </span><span class="tree-label">utils/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   │   └── </span><span class="tree-status">[M]</span> helpers.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseDdl.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowMapper.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> getRenderMapVisitor.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">templates/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> accountsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseTypesPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> eventInstructionClickHouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> eventInstructionPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> instructionsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> instructionsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> lib.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> macros.njk</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">test/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[A]</span> clickhouse-renderer.test.cjs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    └── </span><span class="tree-label">versions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">        └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">            └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
</div></details>
</div></details>
</div></details>
</div>
</div>

<div class="tree-view tree-view-ch-sink">
<div class="tree-code-window">
<details class="tree-node" open><summary><span class="tree-prefix"></span><span class="tree-label">carbon/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">crates/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   └── </span><span class="tree-label">core/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">rows/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> mod.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> admin.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> config.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> http.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> metrics.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> processors.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> retry.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[A]</span> writer.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account_deletion.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> block_details.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> instruction.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> lib.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> pipeline.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> processor.rs</div>
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> transaction.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">datasources/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">rpc-block-crawler-datasource/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">decoders/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> token_ledger_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> best_swap_out_amount_violation_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_quote_error_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_results_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_wsol_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> fee_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> set_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swap_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swaps_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> types.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   ├── </span><span class="tree-status">[M]</span> cpi_event.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       └── </span><span class="tree-label">types/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │           ├── </span><span class="tree-status">[M]</span> candidate_swap.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │           └── </span><span class="tree-status">[M]</span> swap.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> token_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mint.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> multisig.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[M]</span> token.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│           └── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│               ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> amount_to_ui_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> batch_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> close_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> freeze_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> get_account_data_size_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account3_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_immutable_owner_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> revoke_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> set_authority_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> sync_native_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> thaw_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> types.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> ui_amount_to_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> unwrap_lamports_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   └── </span><span class="tree-status">[A]</span> withdraw_excess_lamports_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│               └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">examples/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">prometheus-metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">└── </span><span class="tree-label">packages/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">cli/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │       ├── </span><span class="tree-label">lib/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> decoder.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> prompts.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   └── </span><span class="tree-status">[M]</span> scaffold.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[M]</span> cli.ts</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">renderer/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   │   ├── </span><span class="tree-label">utils/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   │   └── </span><span class="tree-status">[M]</span> helpers.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseDdl.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowMapper.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> getRenderMapVisitor.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">templates/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> accountsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseTypesPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> eventInstructionClickHouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> eventInstructionPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> instructionsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> instructionsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> lib.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> macros.njk</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">test/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[A]</span> clickhouse-renderer.test.cjs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    └── </span><span class="tree-label">versions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">        └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">            └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
</div></details>
</div></details>
</div></details>
</div>
</div>

<div class="tree-view tree-view-decoders">
<div class="tree-code-window">
<details class="tree-node" open><summary><span class="tree-prefix"></span><span class="tree-label">carbon/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">crates/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">core/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">rows/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> mod.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> admin.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> config.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> http.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> metrics.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> processors.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> retry.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[A]</span> writer.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account_deletion.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> block_details.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> instruction.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> lib.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> pipeline.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> processor.rs</div>
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> transaction.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">datasources/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">rpc-block-crawler-datasource/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">├── </span><span class="tree-label">decoders/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> token_ledger_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> best_swap_out_amount_violation_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_quote_error_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_results_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_wsol_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> fee_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> set_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swap_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swaps_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> types.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   ├── </span><span class="tree-status">[M]</span> cpi_event.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">│   │       └── </span><span class="tree-label">types/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │           ├── </span><span class="tree-status">[M]</span> candidate_swap.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │           └── </span><span class="tree-status">[M]</span> swap.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> token_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mint.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> multisig.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[M]</span> token.rs</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">│           └── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">│               ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> amount_to_ui_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> batch_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> close_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> freeze_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> get_account_data_size_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account3_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_immutable_owner_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> revoke_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> set_authority_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> sync_native_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> thaw_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> types.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> ui_amount_to_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> unwrap_lamports_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   └── </span><span class="tree-status">[A]</span> withdraw_excess_lamports_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│               └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">examples/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">prometheus-metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">└── </span><span class="tree-label">packages/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">cli/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │       ├── </span><span class="tree-label">lib/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> decoder.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> prompts.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   └── </span><span class="tree-status">[M]</span> scaffold.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[M]</span> cli.ts</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    ├── </span><span class="tree-label">renderer/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">    │   │   ├── </span><span class="tree-label">utils/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   │   └── </span><span class="tree-status">[M]</span> helpers.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseDdl.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowMapper.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> getRenderMapVisitor.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">templates/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> accountsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseTypesPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> eventInstructionClickHouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> eventInstructionPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> instructionsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> instructionsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> lib.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> macros.njk</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">test/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[A]</span> clickhouse-renderer.test.cjs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    └── </span><span class="tree-label">versions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">        └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">            └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
</div></details>
</div></details>
</div></details>
</div>
</div>

<div class="tree-view tree-view-cli-renderer">
<div class="tree-code-window">
<details class="tree-node" open><summary><span class="tree-prefix"></span><span class="tree-label">carbon/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">crates/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">core/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">rows/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> mod.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> admin.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> config.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> http.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> metrics.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> processors.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[A]</span> retry.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[A]</span> writer.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> account_deletion.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> block_details.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> instruction.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> lib.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> pipeline.rs</div>
<div class="tree-file"><span class="tree-prefix">│           ├── </span><span class="tree-status">[M]</span> processor.rs</div>
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> transaction.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">datasources/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">rpc-block-crawler-datasource/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">decoders/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> token_ledger_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       ├── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │       │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> best_swap_out_amount_violation_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_quote_error_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> candidate_swap_results_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> claim_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_token_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> close_wsol_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> create_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> fee_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> set_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_exact_out_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_v2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> shared_accounts_route_with_token_ledger_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swap_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   ├── </span><span class="tree-status">[A]</span> swaps_event_event_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   │   └── </span><span class="tree-status">[A]</span> types.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│   │       │   ├── </span><span class="tree-status">[M]</span> cpi_event.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │       │   └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   │       └── </span><span class="tree-label">types/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │           ├── </span><span class="tree-status">[M]</span> candidate_swap.rs</div>
<div class="tree-file"><span class="tree-prefix">│   │           └── </span><span class="tree-status">[M]</span> swap.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-decoder/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           ├── </span><span class="tree-label">accounts/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│           │   ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   ├── </span><span class="tree-status">[A]</span> multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   │   └── </span><span class="tree-status">[A]</span> token_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mint.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   ├── </span><span class="tree-status">[M]</span> multisig.rs</div>
<div class="tree-file"><span class="tree-prefix">│           │   └── </span><span class="tree-status">[M]</span> token.rs</div>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│           └── </span><span class="tree-label">instructions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│               ├── </span><span class="tree-label">clickhouse/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> amount_to_ui_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> approve_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> batch_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> burn_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> close_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> freeze_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> get_account_data_size_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account3_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_immutable_owner_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_mint_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig2_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> initialize_multisig_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mint_to_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> mod.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> revoke_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> set_authority_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> sync_native_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> thaw_account_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_checked_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> transfer_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> types.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> ui_amount_to_amount_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   ├── </span><span class="tree-status">[A]</span> unwrap_lamports_row.rs</div>
<div class="tree-file"><span class="tree-prefix">│               │   └── </span><span class="tree-status">[A]</span> withdraw_excess_lamports_row.rs</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">│               └── </span><span class="tree-status">[M]</span> mod.rs</div>
</div></details>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">examples/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   ├── </span><span class="tree-label">jupiter-swap-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│   │       └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">token-program-clickhouse/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[A]</span> main.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">├── </span><span class="tree-label">metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│   └── </span><span class="tree-label">prometheus-metrics/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">│       └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">│           └── </span><span class="tree-status">[M]</span> lib.rs</div>
</div></details>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">└── </span><span class="tree-label">packages/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    ├── </span><span class="tree-label">cli/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    │       ├── </span><span class="tree-label">lib/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> decoder.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   ├── </span><span class="tree-status">[M]</span> prompts.ts</div>
<div class="tree-file"><span class="tree-prefix">    │       │   └── </span><span class="tree-status">[M]</span> scaffold.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[M]</span> cli.ts</div>
</div></details>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">    ├── </span><span class="tree-label">renderer/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<details class="tree-node" open><summary><span class="tree-prefix">    │   │   ├── </span><span class="tree-label">utils/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   │   └── </span><span class="tree-status">[M]</span> helpers.ts</div>
</div></details>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> cargoTomlGenerator.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseDdl.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowMapper.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> getRenderMapVisitor.ts</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">    │   ├── </span><span class="tree-label">templates/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> accountsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> accountsPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> clickhouseTypesPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> eventInstructionClickHouseRowPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> eventInstructionPage.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[A]</span> instructionsClickHouseMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> instructionsMod.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   ├── </span><span class="tree-status">[M]</span> lib.njk</div>
<div class="tree-file"><span class="tree-prefix">    │   │   └── </span><span class="tree-status">[M]</span> macros.njk</div>
</div></details>
<details class="tree-node" open><summary><span class="tree-prefix">    │   └── </span><span class="tree-label">test/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">    │       └── </span><span class="tree-status">[A]</span> clickhouse-renderer.test.cjs</div>
</div></details>
</div></details>
<details class="tree-node"><summary><span class="tree-prefix">    └── </span><span class="tree-label">versions/</span></summary>
<div class="tree-children">
<details class="tree-node"><summary><span class="tree-prefix">        └── </span><span class="tree-label">src/</span></summary>
<div class="tree-children">
<div class="tree-file"><span class="tree-prefix">            └── </span><span class="tree-status">[M]</span> index.ts</div>
</div></details>
</div></details>
</div></details>
</div></details>
</div>
</div>

</div>
