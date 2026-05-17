# Jupiter Swap ClickHouse Example

This example runs real RPC blocks through the generated Jupiter Swap
ClickHouse instruction and CPI-event landing tables. It supports bounded
backfill ranges and a near-head live mode.

## Required Environment

Create `.env` from `.env.example`:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=<provider-rpc-url>
BLOCK_CRAWLER_START_SLOT=<start-slot>
BLOCK_CRAWLER_END_SLOT=<end-slot-or-empty>
BLOCK_CRAWLER_HEAD_LAG_SLOTS=3
PROMETHEUS_METRICS_ADDR=0.0.0.0:9464
LOG_LEVEL=info
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
  pure head-follow mode. The example starts near the current confirmed slot
  using `BLOCK_CRAWLER_HEAD_LAG_SLOTS`.

TokenLedger account fetching is intentionally live-only in this example. The
RPC account read returns the current confirmed account state at fetch time, not
the historical account state at an old swap slot. Reconstructing exact
historical account state requires account-update ingestion or ledger replay and
is outside this thin example. Because of that, the generated TokenLedger
account processor is attached only in pure head-follow mode, when both
`BLOCK_CRAWLER_START_SLOT` and `BLOCK_CRAWLER_END_SLOT` are empty. It is not
attached for bounded backfills or catch-up/tailing runs from an explicit start
slot.

The example exposes Carbon metrics for Prometheus at
`PROMETHEUS_METRICS_ADDR` and keeps log metrics enabled. Use
`monitoring/compose.yaml` to run the local Prometheus/Grafana stack.

To opt into ClickHouse async inserts with `wait_for_async_insert=1`:

```sh
CLICKHOUSE_ASYNC_INSERT=true cargo run -p jupiter-swap-clickhouse-carbon-example
```

This is the production-live ingestion canary path. The default remains
synchronous inserts for deterministic backfills.

## ClickHouse Tables

The example bootstraps one typed landing table per generated Jupiter Swap
instruction family plus one typed CPI/event landing table. In pure head-follow
TokenLedger mode, it also bootstraps the generated TokenLedger account landing
table.
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

<table>
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
      <td>Jupiter program claim steps. These rows mark transactions where the Jupiter program performs a claim action around program-managed state rather than executing a user-facing swap route.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>claim_token_instruction_<wbr>landing</code></td>
      <td>Token claim steps. These rows show tokens being claimed from Jupiter-controlled state into a destination token account, separate from the swap route itself.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>close_token_instruction_<wbr>landing</code></td>
      <td>Token-account cleanup after Jupiter activity. These rows explain transaction tails where a temporary or program-managed token account is closed and, when requested by the instruction, leftover token balance is burned first.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>close_wsol_token_account_<wbr>instruction_landing</code></td>
      <td>Wrapped SOL cleanup. Jupiter may wrap native SOL during a swap and then unwrap it at the end; this table marks the close step for that temporary wrapped-SOL account.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>create_token_account_<wbr>instruction_landing</code></td>
      <td>Token-account setup before or during a swap. These rows show Jupiter creating a supporting token account needed to hold input, output, or intermediate swap funds.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>create_token_ledger_<wbr>instruction_landing</code></td>
      <td>Token-ledger setup. A token ledger lets a later swap step use the actual token amount observed in the transaction instead of a hard-coded input amount.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>exact_out_route_<wbr>instruction_landing</code></td>
      <td>Fixed-output Jupiter swaps. These rows represent trades where the destination amount is fixed first, such as a payment flow, and Jupiter calculates how much source token is needed.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>exact_out_route_v2_<wbr>instruction_landing</code></td>
      <td>Newer fixed-output swap format. It serves the same payment-style purpose as exact-out routing while using Jupiter's newer instruction layout and recording whether the final price moved in the user's favor.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>route_instruction_<wbr>landing</code></td>
      <td>Standard fixed-input Jupiter swaps. These rows show the path Jupiter executed for a known input amount, including the chosen liquidity venues, expected output, slippage guard, and optional platform fee.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>route_v2_instruction_<wbr>landing</code></td>
      <td>Newer fixed-input swap format. It captures the same user-facing trade as the standard route table while using Jupiter's newer route instruction layout.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>route_with_token_ledger_<wbr>instruction_landing</code></td>
      <td>Fixed-input style routes that read their input amount from token-ledger state. These rows are useful when the amount to swap is determined by an earlier step in the same transaction.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>set_token_ledger_<wbr>instruction_landing</code></td>
      <td>Token-ledger update step. This records the token account state that a later ledger-backed Jupiter route uses to determine the actual amount to swap.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>exact_out_route_<wbr>instruction_landing</code></td>
      <td>Fixed-output swaps executed through Jupiter shared accounts. These rows combine payment-style routing with Jupiter-managed intermediate accounts for complex paths.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>exact_out_route_v2_<wbr>instruction_landing</code></td>
      <td>Newer fixed-output shared-account swap format. It records the same kind of payment-style route while using Jupiter's newer shared-account instruction layout.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>route_instruction_<wbr>landing</code></td>
      <td>Standard fixed-input swaps executed through Jupiter shared accounts. These rows are useful for analyzing complex multi-hop trades where Jupiter handles intermediate token accounts.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>route_v2_instruction_<wbr>landing</code></td>
      <td>Newer shared-account fixed-input swap format. It captures the same user-facing shared-account trade while using Jupiter's newer route instruction layout.</td>
    </tr>
    <tr>
      <td><code>jupiter_swap_<wbr>shared_accounts_<wbr>route_with_token_ledger_<wbr>instruction_landing</code></td>
      <td>Shared-account routes that also use token-ledger state. These rows appear when Jupiter combines managed intermediate accounts with an input amount determined earlier in the transaction.</td>
    </tr>
  </tbody>
</table>

CPI/event landing tables:

<table>
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
      <td><code>jupiter_swap_<wbr>cpi_event_<wbr>landing</code></td>
      <td>Jupiter execution events. This table records what happened inside the route after execution starts: fees, individual swap legs, grouped swap legs, quote candidates, quote errors, and cases where the best available output missed the expected threshold.</td>
    </tr>
  </tbody>
</table>

Account landing tables:

<table>
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
      <td>Current confirmed snapshots of Jupiter TokenLedger accounts discovered while following live blocks. These rows help inspect the ledger account state that ledger-backed routes refer to, but they are not historical account-state reconstruction for bounded backfills.</td>
    </tr>
  </tbody>
</table>

Rows include deterministic landing identifiers and common Solana context such
as slot, signature, instruction path, block timing, source name, mode, decoder
version, and ingest time. Landing tables are append-only; repeated backfills can
produce multiple rows for the same logical instruction/event if a slot range is
replayed.
