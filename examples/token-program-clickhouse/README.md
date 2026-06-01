# Token Program ClickHouse Example

This example runs fixed real USDC Token Program account snapshots, then tails
finalized blocks for live Token Program instructions into generated ClickHouse
landing tables.

It fetches four hardwired mainnet USDC accounts with `getMultipleAccounts`:
the USDC mint, the USDC mint authority multisig, the USDC freeze authority
multisig, and one USDC token holding account. All accounts are decoded with
`TokenProgramDecoder` and written through the generated Token Program
ClickHouse account processor.

After the account snapshot, it starts the RPC block crawler at the next
finalized slot, decodes Token Program instructions from finalized blocks, and
writes them through the generated Token Program ClickHouse instruction
processor.

The default path populates all three generated Token Program account landing
tables and at least one generated Token Program instruction landing table
without scanning Token Program accounts or paginating account indexes. The
instruction phase keeps running until the process is stopped. The instruction
phase is block-wide Token Program coverage, not a USDC-only filter; USDC is
used only for the initial account snapshot canary.

## Required Environment

Create `.env` from `.env.example`:

```env
DATABASE_URL=http://carbon:carbon@localhost:8123
RPC_URL=<provider-rpc-url>
PROMETHEUS_METRICS_ADDR=0.0.0.0:9465
LOG_LEVEL=info
```

Use the production/provider RPC URL from the local `.env`. The public
mainnet-beta endpoint is not reliable enough for account and block-crawler
smoke tests. The USDC account set is hardwired in the example.

## Run

```sh
cargo run -p token-program-clickhouse-carbon-example
```

The example uses `getMultipleAccounts` for the fixed account snapshot, then
starts Carbon's RPC block crawler at the next finalized slot and keeps reading
`getBlock` responses for live instruction ingestion. The crawler uses finalized
blocks, binary transaction encoding, version 0 transaction support, and one
in-flight `getBlock` request.

The example exposes Carbon/ClickHouse metrics at `PROMETHEUS_METRICS_ADDR` and
keeps running until interrupted so Prometheus can scrape live counters. Set
`LOG_LEVEL=debug` when you want block-crawler progress logs.

To opt into ClickHouse async inserts with `wait_for_async_insert=1`:

```sh
CLICKHOUSE_ASYNC_INSERT=true cargo run -p token-program-clickhouse-carbon-example
```

Generated ClickHouse table schema is checked before ingestion. The sink creates
missing generated tables, adds missing generated columns, and repairs only
proven enum-extension drift. Other schema drift fails before ingestion.
Destructive drop/recreate cleanup is intentionally not exposed through this
example.

## ClickHouse Tables

The example bootstraps one typed landing table per generated Token Program
account and instruction family. Account rows come from the fixed USDC
`getMultipleAccounts` snapshot. Instruction rows come from decoded Token
Program instructions for the `Tokenkeg...` program in finalized blocks starting
at the slot immediately after that snapshot. Some low-frequency tables may stay
empty in a short run; they are still created because the decoder owns those
instruction families.

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
      <td><code>token_program_<wbr>mint_account_<wbr>landing</code></td>
      <td>The USDC mint account snapshot. This is the token-level state: total supply, decimals, and the authorities that can mint new USDC or freeze USDC token accounts.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>multisig_account_<wbr>landing</code></td>
      <td>The SPL Token multisig accounts currently referenced by the USDC mint as mint authority and freeze authority. These rows show the signer threshold and signer set controlling those USDC authority paths.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>token_account_<wbr>landing</code></td>
      <td>One fixed USDC holding-account snapshot. This is user/protocol balance state for one mint: token owner, raw balance, delegate allowance state, native-WSOL marker when relevant, and close-authority state when present.</td>
    </tr>
  </tbody>
</table>

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
      <td><code>token_program_<wbr>amount_to_ui_amount_<wbr>instruction_landing</code></td>
      <td>On-chain return-data conversions from a raw integer token amount into a UI amount string. These are helper calculations rather than balance changes; for the classic Token Program they reflect mint-decimal formatting, while compatible extension-aware clients use the same pattern when display math cannot be safely guessed off-chain.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>approve_instruction_<wbr>landing</code></td>
      <td>Delegate allowances on token accounts. These rows show an owner authorizing another signer or program to spend or burn up to a raw token amount, a common pattern for escrow, trading, vault, and automation flows.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>approve_checked_<wbr>instruction_landing</code></td>
      <td>Delegate allowances with mint and decimals verification. This checked form protects wallets, hardware signers, and CPI callers from approving the right raw number against the wrong mint or display precision.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>batch_instruction_<wbr>landing</code></td>
      <td>P-token batch payloads. These rows store packed sub-instruction data when newer Token Program flows group multiple token operations into one invocation for lower compute use. A short run can leave this empty if no batched token flow appears in the sampled blocks.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>burn_instruction_<wbr>landing</code></td>
      <td>Raw token burns. These rows record tokens destroyed from a holding account by the owner or delegate, reducing the mint's supply. They are useful for redemption, bridge, treasury, cleanup, and token-lifecycle analysis.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>burn_checked_<wbr>instruction_landing</code></td>
      <td>Token burns with mint and decimals verification. This is the checked burn form used when callers want the burn amount tied explicitly to the intended mint and display precision.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>close_account_<wbr>instruction_landing</code></td>
      <td>Token-account closure and rent recovery. These rows show zero-balance token accounts, temporary wrapped-SOL accounts, or other finished token accounts being closed so their remaining SOL rent balance is returned to a destination account.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>freeze_account_<wbr>instruction_landing</code></td>
      <td>Freeze-authority actions. These rows show a mint's freeze authority locking a token account so its tokens cannot move until a matching thaw operation occurs, a pattern used by managed, compliant, or operationally controlled assets.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>get_account_data_size_<wbr>instruction_landing</code></td>
      <td>Account-size lookup calls. These rows appear when callers ask the Token Program how much account data space is needed before creating a token account. In real block streams they commonly cluster with associated-token-account creation, <code>InitializeImmutableOwner</code>, and <code>InitializeAccount3</code>.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>initialize_account_<wbr>instruction_landing</code></td>
      <td>Classic token-account initialization. These rows show a newly allocated account becoming a token account for exactly one mint and one owner, with the owner and rent sysvar supplied through the account list. This setup must happen atomically with account creation so nobody can take over an uninitialized account.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>initialize_account2_<wbr>instruction_landing</code></td>
      <td>Token-account initialization with the owner carried in instruction data. This variant reduces account-list requirements for CPI callers that do not otherwise need the owner's account meta.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>initialize_account3_<wbr>instruction_landing</code></td>
      <td>Modern token-account initialization with the owner carried in instruction data and no rent sysvar account. This is very common in current associated-token-account creation and wallet/protocol setup flows, especially as an inner instruction emitted by the associated-token-account program.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>initialize_immutable_owner_<wbr>instruction_landing</code></td>
      <td>Immutable-owner setup for token accounts. Associated-token-account flows emit this for Token-2022 compatibility; against the classic Token Program it is effectively a compatibility marker, but it still tells you an ATA-style account creation bundle is happening.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>initialize_mint_<wbr>instruction_landing</code></td>
      <td>Classic mint initialization. These rows define a new token's decimals, mint authority, and optional freeze authority, so they are useful for tracking token launches, supply-control setup, and whether future minting or freezing remains possible.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>initialize_mint2_<wbr>instruction_landing</code></td>
      <td>Mint initialization without the rent sysvar account. These rows carry the same token-launch and authority information as <code>InitializeMint</code> using the newer no-rent-account variant.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>initialize_multisig_<wbr>instruction_landing</code></td>
      <td>Classic multisig authority initialization. These rows define an M-of-N signer set that can act as a mint authority, freeze authority, account owner, delegate, or close authority in later token instructions.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>initialize_multisig2_<wbr>instruction_landing</code></td>
      <td>Multisig initialization without the rent sysvar account. These rows store the same threshold-authority setup as <code>InitializeMultisig</code> using the newer no-rent-account variant.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>mint_to_instruction_<wbr>landing</code></td>
      <td>Raw minting activity. These rows show a mint authority creating new supply and crediting it to a destination token account, which is important for issuance, bridge minting, treasury operations, inflation checks, and supply-monitoring analysis.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>mint_to_checked_<wbr>instruction_landing</code></td>
      <td>Minting activity with decimals verification. This checked form records new token issuance while validating the expected display precision, useful for safer wallet, hardware-signer, and CPI flows.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>revoke_instruction_<wbr>landing</code></td>
      <td>Delegate revocations. These rows show a token account owner removing an existing delegate approval and clearing the remaining delegated allowance from the source account.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>set_authority_<wbr>instruction_landing</code></td>
      <td>Authority changes. These rows capture changes or removals of mint, freeze, account-owner, or close authorities, making them high-signal for token governance, custody, admin-key rotation, and risk monitoring.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>sync_native_<wbr>instruction_landing</code></td>
      <td>Wrapped-SOL synchronization. These rows show native SOL token accounts updating their token <code>amount</code> field after lamports were transferred in, a frequent step in swaps and routes that temporarily wrap SOL.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>thaw_account_<wbr>instruction_landing</code></td>
      <td>Freeze-authority recovery actions. These rows show a mint's freeze authority thawing a frozen token account so transfers, burns, and delegate changes can resume.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>transfer_instruction_<wbr>landing</code></td>
      <td>Raw token transfers. These rows show tokens moving from a source token account to a destination token account, either by the owner or by an approved delegate. Legacy clients and CPIs still emit this form.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>transfer_checked_<wbr>instruction_landing</code></td>
      <td>Token transfers with mint and decimals verification. This is the preferred high-signal transfer table for user and protocol movement because it carries the mint and expected display precision in the instruction payload.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>ui_amount_to_amount_<wbr>instruction_landing</code></td>
      <td>On-chain return-data conversions from a UI amount string into a raw integer token amount for a mint. These are helper calculations, useful when extension-aware amount conversion must happen through the Token Program instead of a client-side decimals rule.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>unwrap_lamports_<wbr>instruction_landing</code></td>
      <td>P-token native-SOL unwrap helper calls. These rows represent newer paths that transfer a requested amount, or all excess over rent exemption, from a wrapped/native SOL token account to any destination account without requiring the older temporary-account cleanup pattern.</td>
    </tr>
    <tr>
      <td><code>token_program_<wbr>withdraw_excess_lamports_<wbr>instruction_landing</code></td>
      <td>P-token excess-lamport recovery. These rows show SOL being rescued from token-owned accounts, such as mint or multisig accounts with accidental excess lamports, down to their rent-exempt reserve without changing token balances or mint supply.</td>
    </tr>
  </tbody>
</table>

`account_id` is a deterministic landing identifier based on program, pubkey,
slot, and account type. Landing rows are append-only snapshots. If the same
token account is fetched again at a later slot, the table will contain another
row for the same `pubkey` with a different `slot` and `account_id`.

`transaction_signature` is expected to be `NULL` because these snapshots are
current account reads, not transaction-scoped account updates.

Instruction rows use deterministic instruction identifiers based on the
transaction signature, slot, and instruction path. They include common Solana
transaction context plus the typed instruction payload, such as transfer amount
and decimals for `TransferChecked`.
