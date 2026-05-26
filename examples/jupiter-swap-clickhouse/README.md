# Jupiter Swap ClickHouse Example

This example runs real RPC blocks through the generated Jupiter Swap
ClickHouse instruction and CPI-event landing tables. It supports bounded
backfill ranges, explicit start-slot catch-up/tailing, and pure head-follow
mode.

## Required Environment

Create `.env` from `.env.example`:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=<provider-rpc-url>
BLOCK_CRAWLER_START_SLOT=<start-slot>
BLOCK_CRAWLER_END_SLOT=<end-slot-or-empty>
BLOCK_CRAWLER_HEAD_LAG_SLOTS=3
PROMETHEUS_METRICS_ADDR=0.0.0.0:9464
LOG_LEVEL=debug
```

Use the production/provider RPC URL from the local `.env`. The public
mainnet-beta endpoint is not reliable enough for this smoke test.

## Run

```sh
cargo run -p jupiter-swap-clickhouse-carbon-example
```

Mode is inferred from the slot env:

- Set both `BLOCK_CRAWLER_START_SLOT` and `BLOCK_CRAWLER_END_SLOT` for a
  bounded backfill.
- Set `BLOCK_CRAWLER_START_SLOT` and leave `BLOCK_CRAWLER_END_SLOT` empty to
  catch up from that slot and then keep following head.
- Leave both `BLOCK_CRAWLER_START_SLOT` and `BLOCK_CRAWLER_END_SLOT` empty for
  pure head-follow mode. The example starts near the current finalized slot
  using `BLOCK_CRAWLER_HEAD_LAG_SLOTS`.

The block crawler uses finalized blocks, binary transaction encoding, version
0 transaction support, and one in-flight `getBlock` request. This keeps the
example conservative and provider-friendly; it is not tuned as a high-throughput
catch-up worker.

TokenLedger account fetching is intentionally live-only in this example. The
RPC account read returns the current confirmed account state at fetch time, not
the historical account state at an old swap slot. Reconstructing exact
historical account state requires account-update ingestion or ledger replay and
is outside this thin example. Because of that, the generated TokenLedger
account processor is attached only in pure head-follow mode, when both
`BLOCK_CRAWLER_START_SLOT` and `BLOCK_CRAWLER_END_SLOT` are empty. It is not
attached for bounded backfills or catch-up/tailing runs from an explicit start
slot. In that mode, first-seen TokenLedger pubkeys are fetched immediately with
`getAccountInfo` and written through the generated Jupiter account processor.

The example exposes Carbon metrics for Prometheus at
`PROMETHEUS_METRICS_ADDR` and keeps log metrics enabled. Use
`monitoring/compose.yaml` to run the local Prometheus/Grafana stack.

To opt into ClickHouse async inserts with `wait_for_async_insert=1`:

```sh
CLICKHOUSE_ASYNC_INSERT=true cargo run -p jupiter-swap-clickhouse-carbon-example
```

This is the production-live ingestion canary path. The default remains
synchronous inserts for deterministic backfills.

Generated ClickHouse table schema is checked before ingestion. The sink creates
missing generated tables, adds missing generated columns, and repairs only
proven enum-extension drift such as newly generated Jupiter swap enum variants
on an older local table. Other schema drift fails before ingestion. Destructive
drop/recreate cleanup is intentionally not exposed through this example.

## ClickHouse Tables

The example bootstraps one typed landing table per generated Jupiter Swap
instruction family and one typed landing table per generated CPI/event family.
In pure head-follow TokenLedger mode, it also bootstraps the generated
TokenLedger account landing table.
These tables are append-only landing records of decoded Jupiter activity, with
enough Solana context to replay, trace, and analyze the decoded instruction,
event, or fetched account snapshot later. Empty instruction tables are expected
when the selected slot range does not contain that instruction type.

In Jupiter terms, a route is the swap path Jupiter chose after comparing Solana
liquidity sources. It can be a direct swap or a split/multi-hop path. Exact-out
tables cover payment-style swaps where the receiver gets a fixed output amount
and Jupiter works out the required input amount. Shared-account tables cover
routes where Jupiter uses its own intermediate token accounts for complex paths,
so the user does not need to create every temporary account.

A few Jupiter instruction structs are payload-free. Those rows are not missing
data; the decoded instruction has no instruction-data fields, so the table
exists to record that the instruction occurred in the selected transaction path.

Instruction landing tables:

<table width="100%">
  <colgroup>
    <col width="34%" />
    <col width="66%" />
  </colgroup>
  <thead>
    <tr>
      <th>Table</th>
      <th>What it stores</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>jupiter_swap_<wbr>claim_instruction_<wbr>landing</code></td>
      <td>Jupiter-controlled claim activity. These rows represent admin-style claim steps around program-managed balances or state, not ordinary user swap routing.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>claim_token_instruction_<wbr>landing</code></td>
      <td>Token commission, referral, or revenue claim activity. These rows show tokens moving out of Jupiter-controlled program token accounts into destination token accounts, separate from the swap route itself.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>close_token_instruction_<wbr>landing</code></td>
      <td>Operator or admin cleanup of Jupiter program token accounts. These rows explain maintenance flows where a program-managed token account is closed and, when requested by the instruction, leftover token balance is burned first.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>close_wsol_token_account_<wbr>instruction_landing</code></td>
      <td>Jupiter-specific wrapped-SOL cleanup. This table marks Jupiter's own WSOL account close helper, which is separate from ordinary SPL Token <code>CloseAccount</code> cleanup that can also appear in swap transactions.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>create_token_account_<wbr>instruction_landing</code></td>
      <td>Jupiter token-account setup. These rows show Jupiter creating a supporting token account for a user, mint, and token program before the route needs that account for swap settlement or intermediate funds.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>create_token_ledger_<wbr>instruction_landing</code></td>
      <td>TokenLedger account creation and initialization. This is a rare setup path for ledger-backed swaps; normal <code>useTokenLedger=true</code> execution usually appears as <code>SetTokenLedger</code> plus <code>RouteWithTokenLedger</code> or <code>SharedAccountsRouteWithTokenLedger</code>.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>exact_out_route_<wbr>instruction_landing</code></td>
      <td>Fixed-output Jupiter routes. These rows represent swaps where the desired output amount is fixed first, such as payment or dust-sweep flows, and Jupiter records the route plan, quoted input amount, slippage guard, and optional platform fee.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>exact_out_route_v2_<wbr>instruction_landing</code></td>
      <td>Newer fixed-output route format. It stores the same payment-style route as exact-out, with the V2 layout used for newer token-program handling and positive-slippage reporting.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>route_instruction_<wbr>landing</code></td>
      <td>Standard fixed-input Jupiter routes. These high-frequency rows show the path Jupiter executed for a known input amount, including selected liquidity venues, split percentages, expected output, slippage guard, and optional platform fee.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>route_v2_instruction_<wbr>landing</code></td>
      <td>Newer fixed-input route format. It captures the same user-facing swap as the standard route table, with the V2 layout used for newer token-program handling, basis-point route splits, and positive-slippage reporting.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>route_with_token_ledger_<wbr>instruction_landing</code></td>
      <td>Ledger-backed Jupiter routes. These rows appear when the swap amount is determined by an earlier instruction in the same transaction; the route reads TokenLedger state instead of carrying a fixed input amount directly.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>set_token_ledger_<wbr>instruction_landing</code></td>
      <td>TokenLedger update step. This records the source token account balance that a later ledger-backed Jupiter route uses to calculate the actual amount added or swapped inside the same transaction.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>exact_out_route_<wbr>instruction_landing</code></td>
      <td>Fixed-output routes using Jupiter shared accounts. These rows combine exact-output payment-style routing with Jupiter-managed intermediate accounts, reducing the need for users or integrators to create every intermediate token account themselves.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>exact_out_route_v2_<wbr>instruction_landing</code></td>
      <td>Newer fixed-output shared-account route format. This is a real but low-frequency on-chain path; it stores exact-output routes that use Jupiter shared accounts plus the V2 layout for newer token-program handling and positive-slippage reporting.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>route_instruction_<wbr>landing</code></td>
      <td>Standard fixed-input routes using Jupiter shared accounts. These high-frequency rows are common in wallet and integrator flows where Jupiter handles intermediate accounts for complex multi-hop trades.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>route_v2_instruction_<wbr>landing</code></td>
      <td>Newer shared-account fixed-input route format. It stores the same managed-intermediate-account trade as shared-account routes, with the V2 layout used for newer token-program handling, basis-point route splits, and positive-slippage reporting.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>route_with_token_ledger_<wbr>instruction_landing</code></td>
      <td>Shared-account routes that also use TokenLedger state. These rows appear when Jupiter combines managed intermediate accounts with an input amount determined earlier in the same transaction.</td>
    </tr>
  </tbody>
</table>

CPI/event landing tables:

<table width="100%">
  <colgroup>
    <col width="34%" />
    <col width="66%" />
  </colgroup>
  <thead>
    <tr>
      <th>Table</th>
      <th>What it stores</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>jupiter_swap_<wbr>fee_event_<wbr>landing</code></td>
      <td>Fee events emitted while Jupiter executes a route. These rows identify the fee account, fee mint, and collected amount so platform, referral, or protocol-fee activity can be analyzed separately from the swap instruction payload.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>swap_event_<wbr>landing</code></td>
      <td>One executed swap leg inside a Jupiter route. These rows show the AMM or liquidity venue plus the input/output mints and amounts for that leg, which is the event-side execution record for a concrete route hop.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>swaps_event_<wbr>landing</code></td>
      <td>Grouped swap-leg output emitted by newer Jupiter execution paths. Each row stores the route's collection of executed swap legs together, making it easier to inspect multi-hop or split execution as one CPI/event record.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>candidate_swap_<wbr>results_landing</code></td>
      <td>Dynamic-route candidate outcomes. These rows capture Jupiter's internal comparison of candidate routes as either a successful output amount or a program error, which helps explain why one route candidate was selected or rejected.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>candidate_swap_<wbr>quote_error_landing</code></td>
      <td>Quote failures for dynamic-route candidates. These rows record which candidate failed to quote, the attempted input amount, and the quote error code, so failed candidate evaluation can be analyzed without treating the whole transaction as failed.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>best_swap_out_amount_<wbr>violation_landing</code></td>
      <td>Output-guardrail violations from dynamic route evaluation. These rows compare Jupiter's expected best output with the actual available output when the best candidate misses the required output threshold.</td>
    </tr>
  </tbody>
</table>

Account landing tables:

<table width="100%">
  <colgroup>
    <col width="34%" />
    <col width="66%" />
  </colgroup>
  <thead>
    <tr>
      <th>Table</th>
      <th>What it stores</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>jupiter_swap_<wbr>token_ledger_<wbr>account_landing</code></td>
      <td>Current confirmed snapshots of Jupiter TokenLedger accounts discovered while following live blocks. Each row stores the ledger's tracked token account and amount so ledger-backed routes can be inspected alongside the account state they reference; this is not historical account-state reconstruction for bounded backfills.</td>
    </tr>
  </tbody>
</table>

Rows include deterministic landing identifiers and common Solana context such
as slot, signature, instruction path, block timing, source name, mode, decoder
version, and ingest time. Landing tables are append-only; repeated backfills can
produce multiple rows for the same logical instruction/event if a slot range is
replayed.
