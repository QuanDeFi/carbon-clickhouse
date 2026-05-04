use carbon_core::{
    clickhouse::rows::{
        deterministic_instruction_id, ClickHouseRow, ClickHouseRowContext, ClickHouseTable,
    },
    instruction::InstructionMetadata,
};
use chrono::{DateTime, Utc};

use crate::PROGRAM_ID as JUPITER_SWAP_PROGRAM_ID;

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct JupiterSwapInstructionLandingMetadata {
    pub program_id: String,
    pub family_name: String,
    pub instruction_type: String,
    pub instruction_id: String,
    pub slot: u64,
    pub signature: String,
    pub instruction_index: u32,
    pub stack_height: u32,
    pub absolute_path: Vec<u8>,
    pub source_name: String,
    pub mode: String,
    pub decoder_version: String,
    pub ingest_ts: String,
    pub chain_time: Option<String>,
    pub partition_time: String,
    pub block_hash: Option<String>,
    pub tx_index: Option<u64>,
}

impl JupiterSwapInstructionLandingMetadata {
    pub fn from_parts(
        family_name: &str,
        instruction_type: &str,
        metadata: &InstructionMetadata,
        context: &ClickHouseRowContext,
    ) -> Self {
        Self::from_parts_with_ingest_ts(
            family_name,
            instruction_type,
            metadata,
            context,
            Utc::now(),
        )
    }

    pub fn from_parts_with_ingest_ts(
        family_name: &str,
        instruction_type: &str,
        metadata: &InstructionMetadata,
        context: &ClickHouseRowContext,
        ingest_ts: DateTime<Utc>,
    ) -> Self {
        let chain_time = metadata
            .transaction_metadata
            .block_time
            .and_then(DateTime::<Utc>::from_timestamp_secs);
        let partition_time = chain_time.unwrap_or(ingest_ts);
        let signature = metadata.transaction_metadata.signature.to_string();

        Self {
            program_id: JUPITER_SWAP_PROGRAM_ID.to_string(),
            family_name: family_name.to_string(),
            instruction_type: instruction_type.to_string(),
            instruction_id: deterministic_instruction_id(
                JUPITER_SWAP_PROGRAM_ID.as_ref(),
                &signature,
                &metadata.absolute_path,
                instruction_type,
            ),
            slot: metadata.transaction_metadata.slot,
            signature,
            instruction_index: metadata.index,
            stack_height: metadata.stack_height,
            absolute_path: metadata.absolute_path.clone(),
            source_name: context.source_name.clone(),
            mode: context.mode.clone(),
            decoder_version: context.decoder_version.clone(),
            ingest_ts: format_datetime(ingest_ts),
            chain_time: chain_time.map(format_datetime),
            partition_time: format_datetime(partition_time),
            block_hash: metadata
                .transaction_metadata
                .block_hash
                .map(|hash| hash.to_string()),
            tx_index: metadata.transaction_metadata.index,
        }
    }

    pub fn partition_key(&self) -> String {
        self.partition_time[..4].to_string()
    }
}

const INSTRUCTION_METADATA_COLUMNS: &[&str] = &[
    "program_id",
    "family_name",
    "instruction_type",
    "instruction_id",
    "slot",
    "signature",
    "instruction_index",
    "stack_height",
    "absolute_path",
    "source_name",
    "mode",
    "decoder_version",
    "ingest_ts",
    "chain_time",
    "partition_time",
    "block_hash",
    "tx_index",
];

fn instruction_columns(payload_columns: &[&'static str]) -> Vec<&'static str> {
    let mut columns = INSTRUCTION_METADATA_COLUMNS.to_vec();
    columns.extend(payload_columns);
    columns
}

fn create_instruction_table_sql(table_name: &str, payload_columns: &[String]) -> String {
    let payload_sql = if payload_columns.is_empty() {
        String::new()
    } else {
        format!(",{}", payload_columns.join(","))
    };

    format!(
        "CREATE TABLE IF NOT EXISTS {table_name} (\
            program_id String,\
            family_name String,\
            instruction_type String,\
            instruction_id String,\
            slot UInt64,\
            signature String,\
            instruction_index UInt32,\
            stack_height UInt32,\
            absolute_path Array(UInt8),\
            source_name String,\
            mode String,\
            decoder_version String,\
            ingest_ts DateTime64(3, 'UTC'),\
            chain_time Nullable(DateTime64(3, 'UTC')),\
            partition_time DateTime64(3, 'UTC'),\
            block_hash Nullable(String),\
            tx_index Nullable(UInt64){payload_sql}\
        ) ENGINE = MergeTree \
        PARTITION BY toYear(partition_time) \
        ORDER BY (program_id, family_name, instruction_id, slot)"
    )
}

fn format_datetime(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

fn route_plan_step_clickhouse_type(is_v2: bool) -> String {
    let split_column = if is_v2 { "bps UInt16" } else { "percent UInt8" };
    format!(
        "Array(Tuple(swap {}, {split_column}, input_index UInt8, output_index UInt8))",
        swap_clickhouse_type()
    )
}

fn swap_clickhouse_type() -> String {
    format!(
        "Tuple(\
            variant LowCardinality(String),\
            side Nullable(Enum8('Bid' = 0, 'Ask' = 1)),\
            hylo_swap_type Nullable({hylo_swap_type}),\
            a_to_b Nullable(Bool),\
            x_to_y Nullable(Bool),\
            stable Nullable(Bool),\
            from_token_id Nullable(UInt64),\
            to_token_id Nullable(UInt64),\
            bridge_stake_seed Nullable(UInt32),\
            pool_index Nullable(UInt8),\
            quantity_is_input Nullable(Bool),\
            quantity_is_collateral Nullable(Bool),\
            src_lst_value_calc_accs Nullable(UInt8),\
            dst_lst_value_calc_accs Nullable(UInt8),\
            src_lst_index Nullable(UInt32),\
            dst_lst_index Nullable(UInt32),\
            lst_value_calc_accs Nullable(UInt8),\
            lst_index Nullable(UInt32),\
            remaining_accounts_info_present Bool,\
            remaining_accounts_slices Array(Tuple(accounts_type UInt8, length UInt8)),\
            is_y Nullable(Bool),\
            is_quote_to_base Nullable(Bool),\
            in_index Nullable(UInt8),\
            out_index Nullable(UInt8),\
            share_fee_rate Nullable(UInt64),\
            is_bid Nullable(Bool),\
            blacklist_bump Nullable(UInt8),\
            swap_id Nullable(UInt128),\
            is_base_to_quote Nullable(Bool),\
            candidate_swaps Array({candidate_swap}),\
            candidate_swaps_with_bps Array(Tuple(candidate_swap {candidate_swap}, bps UInt32)),\
            best_position Nullable(UInt8),\
            is_mint Nullable(Bool),\
            fill_data Array(UInt8),\
            lst_amounts Array(UInt64),\
            seed Nullable(UInt64),\
            amount_is_token_a Nullable(Bool),\
            is_base_in Nullable(Bool),\
            auth_amount_in Nullable(UInt64),\
            auth Nullable(UInt64),\
            swap_for_y Nullable(Bool),\
            max_split_quote_calls Nullable(UInt8),\
            max_split_candidates Nullable(UInt8))",
        candidate_swap = candidate_swap_clickhouse_type(),
        hylo_swap_type = hylo_swap_type_clickhouse_type()
    )
}

fn candidate_swap_clickhouse_type() -> &'static str {
    "Tuple(\
        variant LowCardinality(String),\
        side Nullable(Enum8('Bid' = 0, 'Ask' = 1)),\
        swap_id Nullable(UInt64),\
        is_base_to_quote Nullable(Bool),\
        a_to_b Nullable(Bool),\
        is_bid Nullable(Bool))"
}

fn hylo_swap_type_clickhouse_type() -> &'static str {
    "Enum8(\
        'MintStable' = 0,\
        'RedeemStable' = 1,\
        'MintLever' = 2,\
        'RedeemLever' = 3,\
        'SwapStableToLever' = 4,\
        'SwapLeverToStable' = 5,\
        'StabilityPoolDeposit' = 6,\
        'StabilityPoolWithdraw' = 7)"
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ClickHouseRemainingAccountsSlice {
    pub accounts_type: u8,
    pub length: u8,
}

impl From<&crate::types::RemainingAccountsSlice> for ClickHouseRemainingAccountsSlice {
    fn from(value: &crate::types::RemainingAccountsSlice) -> Self {
        Self {
            accounts_type: value.accounts_type,
            length: value.length,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ClickHouseCandidateSwap {
    pub variant: String,
    pub side: Option<String>,
    pub swap_id: Option<u64>,
    pub is_base_to_quote: Option<bool>,
    pub a_to_b: Option<bool>,
    pub is_bid: Option<bool>,
}

impl From<&crate::types::CandidateSwap> for ClickHouseCandidateSwap {
    fn from(value: &crate::types::CandidateSwap) -> Self {
        let mut candidate = Self {
            variant: clickhouse_enum_variant(value),
            side: None,
            swap_id: None,
            is_base_to_quote: None,
            a_to_b: None,
            is_bid: None,
        };

        match value {
            crate::types::CandidateSwap::HumidiFi {
                swap_id,
                is_base_to_quote,
            }
            | crate::types::CandidateSwap::HumidiFiV2 {
                swap_id,
                is_base_to_quote,
            } => {
                candidate.swap_id = Some(*swap_id);
                candidate.is_base_to_quote = Some(*is_base_to_quote);
            }
            crate::types::CandidateSwap::TesseraV { side } => {
                candidate.side = Some(clickhouse_enum_variant(side));
            }
            crate::types::CandidateSwap::Whirlpool { a_to_b }
            | crate::types::CandidateSwap::BisonFiV2 { a_to_b } => {
                candidate.a_to_b = Some(*a_to_b);
            }
            crate::types::CandidateSwap::GoonFiV2 { is_bid } => {
                candidate.is_bid = Some(*is_bid);
            }
            crate::types::CandidateSwap::RaydiumV2
            | crate::types::CandidateSwap::RaydiumClmm
            | crate::types::CandidateSwap::ZeroFi => {}
        }

        candidate
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ClickHouseCandidateSwapWithBps {
    pub candidate_swap: ClickHouseCandidateSwap,
    pub bps: u32,
}

impl From<&crate::types::CandidateSwapWithBps> for ClickHouseCandidateSwapWithBps {
    fn from(value: &crate::types::CandidateSwapWithBps) -> Self {
        Self {
            candidate_swap: ClickHouseCandidateSwap::from(&value.candidate_swap),
            bps: value.bps,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseUInt128(pub u128);

impl serde::Serialize for ClickHouseUInt128 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ClickHouseSwap {
    pub variant: String,
    pub side: Option<String>,
    pub hylo_swap_type: Option<String>,
    pub a_to_b: Option<bool>,
    pub x_to_y: Option<bool>,
    pub stable: Option<bool>,
    pub from_token_id: Option<u64>,
    pub to_token_id: Option<u64>,
    pub bridge_stake_seed: Option<u32>,
    pub pool_index: Option<u8>,
    pub quantity_is_input: Option<bool>,
    pub quantity_is_collateral: Option<bool>,
    pub src_lst_value_calc_accs: Option<u8>,
    pub dst_lst_value_calc_accs: Option<u8>,
    pub src_lst_index: Option<u32>,
    pub dst_lst_index: Option<u32>,
    pub lst_value_calc_accs: Option<u8>,
    pub lst_index: Option<u32>,
    pub remaining_accounts_info_present: bool,
    pub remaining_accounts_slices: Vec<ClickHouseRemainingAccountsSlice>,
    pub is_y: Option<bool>,
    pub is_quote_to_base: Option<bool>,
    pub in_index: Option<u8>,
    pub out_index: Option<u8>,
    pub share_fee_rate: Option<u64>,
    pub is_bid: Option<bool>,
    pub blacklist_bump: Option<u8>,
    pub swap_id: Option<ClickHouseUInt128>,
    pub is_base_to_quote: Option<bool>,
    pub candidate_swaps: Vec<ClickHouseCandidateSwap>,
    pub candidate_swaps_with_bps: Vec<ClickHouseCandidateSwapWithBps>,
    pub best_position: Option<u8>,
    pub is_mint: Option<bool>,
    pub fill_data: Vec<u8>,
    pub lst_amounts: Vec<u64>,
    pub seed: Option<u64>,
    pub amount_is_token_a: Option<bool>,
    pub is_base_in: Option<bool>,
    pub auth_amount_in: Option<u64>,
    pub auth: Option<u64>,
    pub swap_for_y: Option<bool>,
    pub max_split_quote_calls: Option<u8>,
    pub max_split_candidates: Option<u8>,
}

impl ClickHouseSwap {
    fn new(variant: String) -> Self {
        Self {
            variant,
            side: None,
            hylo_swap_type: None,
            a_to_b: None,
            x_to_y: None,
            stable: None,
            from_token_id: None,
            to_token_id: None,
            bridge_stake_seed: None,
            pool_index: None,
            quantity_is_input: None,
            quantity_is_collateral: None,
            src_lst_value_calc_accs: None,
            dst_lst_value_calc_accs: None,
            src_lst_index: None,
            dst_lst_index: None,
            lst_value_calc_accs: None,
            lst_index: None,
            remaining_accounts_info_present: false,
            remaining_accounts_slices: Vec::new(),
            is_y: None,
            is_quote_to_base: None,
            in_index: None,
            out_index: None,
            share_fee_rate: None,
            is_bid: None,
            blacklist_bump: None,
            swap_id: None,
            is_base_to_quote: None,
            candidate_swaps: Vec::new(),
            candidate_swaps_with_bps: Vec::new(),
            best_position: None,
            is_mint: None,
            fill_data: Vec::new(),
            lst_amounts: Vec::new(),
            seed: None,
            amount_is_token_a: None,
            is_base_in: None,
            auth_amount_in: None,
            auth: None,
            swap_for_y: None,
            max_split_quote_calls: None,
            max_split_candidates: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ClickHouseRoutePlanStep {
    pub swap: ClickHouseSwap,
    pub percent: u8,
    pub input_index: u8,
    pub output_index: u8,
}

impl From<&crate::types::RoutePlanStep> for ClickHouseRoutePlanStep {
    fn from(value: &crate::types::RoutePlanStep) -> Self {
        Self {
            swap: clickhouse_swap(&value.swap),
            percent: value.percent,
            input_index: value.input_index,
            output_index: value.output_index,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ClickHouseRoutePlanStepV2 {
    pub swap: ClickHouseSwap,
    pub bps: u16,
    pub input_index: u8,
    pub output_index: u8,
}

impl From<&crate::types::RoutePlanStepV2> for ClickHouseRoutePlanStepV2 {
    fn from(value: &crate::types::RoutePlanStepV2) -> Self {
        Self {
            swap: clickhouse_swap(&value.swap),
            bps: value.bps,
            input_index: value.input_index,
            output_index: value.output_index,
        }
    }
}

fn clickhouse_route_plan(value: &[crate::types::RoutePlanStep]) -> Vec<ClickHouseRoutePlanStep> {
    value.iter().map(ClickHouseRoutePlanStep::from).collect()
}

fn clickhouse_route_plan_v2(
    value: &[crate::types::RoutePlanStepV2],
) -> Vec<ClickHouseRoutePlanStepV2> {
    value.iter().map(ClickHouseRoutePlanStepV2::from).collect()
}

fn clickhouse_swap(value: &crate::types::Swap) -> ClickHouseSwap {
    let mut swap = ClickHouseSwap::new(clickhouse_enum_variant(value));

    match value {
        crate::types::Swap::WhirlpoolSwapV2 {
            a_to_b,
            remaining_accounts_info,
        }
        | crate::types::Swap::DefiTuna {
            a_to_b,
            remaining_accounts_info,
        } => {
            swap.a_to_b = Some(*a_to_b);
            if let Some(remaining_accounts_info) = remaining_accounts_info {
                swap.remaining_accounts_info_present = true;
                swap.remaining_accounts_slices =
                    clickhouse_remaining_accounts_slices(remaining_accounts_info);
            }
        }
        crate::types::Swap::JupiterRfqV2 { side, fill_data } => {
            swap.side = Some(clickhouse_enum_variant(side));
            swap.fill_data = fill_data.clone();
        }
        crate::types::Swap::WhaleStreetV2 {
            side,
            auth_amount_in,
            auth,
        } => {
            swap.side = Some(clickhouse_enum_variant(side));
            swap.auth_amount_in = Some(*auth_amount_in);
            swap.auth = Some(*auth);
        }
        crate::types::Swap::Crema { a_to_b }
        | crate::types::Swap::Whirlpool { a_to_b }
        | crate::types::Swap::Heaven { a_to_b }
        | crate::types::Swap::AlphaQ { a_to_b }
        | crate::types::Swap::BisonFi { a_to_b }
        | crate::types::Swap::BisonFiV2 { a_to_b } => {
            swap.a_to_b = Some(*a_to_b);
        }
        crate::types::Swap::Serum { side }
        | crate::types::Swap::Aldrin { side }
        | crate::types::Swap::AldrinV2 { side }
        | crate::types::Swap::Dradex { side }
        | crate::types::Swap::Openbook { side }
        | crate::types::Swap::Phoenix { side }
        | crate::types::Swap::OpenBookV2 { side }
        | crate::types::Swap::Plasma { side }
        | crate::types::Swap::TesseraV { side }
        | crate::types::Swap::Futarchy { side }
        | crate::types::Swap::WhaleStreet { side }
        | crate::types::Swap::Manifest { side }
        | crate::types::Swap::Quantum { side } => {
            swap.side = Some(clickhouse_enum_variant(side));
        }
        crate::types::Swap::Hylo { swap_type } => {
            swap.hylo_swap_type = Some(clickhouse_enum_variant(swap_type));
        }
        crate::types::Swap::Invariant { x_to_y } | crate::types::Swap::MarcoPolo { x_to_y } => {
            swap.x_to_y = Some(*x_to_y);
        }
        crate::types::Swap::DeltaFi { stable } => {
            swap.stable = Some(*stable);
        }
        crate::types::Swap::Symmetry {
            from_token_id,
            to_token_id,
        } => {
            swap.from_token_id = Some(*from_token_id);
            swap.to_token_id = Some(*to_token_id);
        }
        crate::types::Swap::StakeDexSwapViaStake { bridge_stake_seed }
        | crate::types::Swap::StakeDexPrefundWithdrawStakeAndDepositStake { bridge_stake_seed } => {
            swap.bridge_stake_seed = Some(*bridge_stake_seed);
        }
        crate::types::Swap::Clone {
            pool_index,
            quantity_is_input,
            quantity_is_collateral,
        } => {
            swap.pool_index = Some(*pool_index);
            swap.quantity_is_input = Some(*quantity_is_input);
            swap.quantity_is_collateral = Some(*quantity_is_collateral);
        }
        crate::types::Swap::SanctumS {
            src_lst_value_calc_accs,
            dst_lst_value_calc_accs,
            src_lst_index,
            dst_lst_index,
        }
        | crate::types::Swap::SanctumSV2 {
            src_lst_value_calc_accs,
            dst_lst_value_calc_accs,
            src_lst_index,
            dst_lst_index,
        } => {
            swap.src_lst_value_calc_accs = Some(*src_lst_value_calc_accs);
            swap.dst_lst_value_calc_accs = Some(*dst_lst_value_calc_accs);
            swap.src_lst_index = Some(*src_lst_index);
            swap.dst_lst_index = Some(*dst_lst_index);
        }
        crate::types::Swap::SanctumSAddLiquidity {
            lst_value_calc_accs,
            lst_index,
        }
        | crate::types::Swap::SanctumSRemoveLiquidity {
            lst_value_calc_accs,
            lst_index,
        } => {
            swap.lst_value_calc_accs = Some(*lst_value_calc_accs);
            swap.lst_index = Some(*lst_index);
        }
        crate::types::Swap::MeteoraDlmmSwapV2 {
            remaining_accounts_info,
        } => {
            swap.remaining_accounts_info_present = true;
            swap.remaining_accounts_slices =
                clickhouse_remaining_accounts_slices(remaining_accounts_info);
        }
        crate::types::Swap::Obric { x_to_y } => {
            swap.x_to_y = Some(*x_to_y);
        }
        crate::types::Swap::FoxClaimPartial { is_y } => {
            swap.is_y = Some(*is_y);
        }
        crate::types::Swap::SolFi { is_quote_to_base }
        | crate::types::Swap::SolFiV2 { is_quote_to_base } => {
            swap.is_quote_to_base = Some(*is_quote_to_base);
        }
        crate::types::Swap::TokenMill { side } => {
            swap.side = Some(clickhouse_enum_variant(side));
        }
        crate::types::Swap::Perena {
            in_index,
            out_index,
        } => {
            swap.in_index = Some(*in_index);
            swap.out_index = Some(*out_index);
        }
        crate::types::Swap::RaydiumLaunchlabBuy { share_fee_rate }
        | crate::types::Swap::RaydiumLaunchlabSell { share_fee_rate } => {
            swap.share_fee_rate = Some(*share_fee_rate);
        }
        crate::types::Swap::GoonFi {
            is_bid,
            blacklist_bump,
        } => {
            swap.is_bid = Some(*is_bid);
            swap.blacklist_bump = Some(*blacklist_bump);
        }
        crate::types::Swap::GoonFiV2 { is_bid } => {
            swap.is_bid = Some(*is_bid);
        }
        crate::types::Swap::HumidiFi {
            swap_id,
            is_base_to_quote,
        }
        | crate::types::Swap::HumidiFiV2 {
            swap_id,
            is_base_to_quote,
        } => {
            swap.swap_id = Some(ClickHouseUInt128((*swap_id).into()));
            swap.is_base_to_quote = Some(*is_base_to_quote);
        }
        crate::types::Swap::Scorch { swap_id } => {
            swap.swap_id = Some(ClickHouseUInt128(*swap_id));
        }
        crate::types::Swap::SarosDlmm { swap_for_y } => {
            swap.swap_for_y = Some(*swap_for_y);
        }
        crate::types::Swap::DynamicV1 {
            candidate_swaps,
            best_position,
        } => {
            swap.candidate_swaps = candidate_swaps
                .iter()
                .map(ClickHouseCandidateSwap::from)
                .collect();
            swap.best_position = *best_position;
        }
        crate::types::Swap::PerenaStar { is_mint } => {
            swap.is_mint = Some(*is_mint);
        }
        crate::types::Swap::VaultLiquidUnstake { lst_amounts, seed } => {
            swap.lst_amounts = lst_amounts.to_vec();
            swap.seed = Some(*seed);
        }
        crate::types::Swap::Riptide { amount_is_token_a } => {
            swap.amount_is_token_a = Some(*amount_is_token_a);
        }
        crate::types::Swap::TaurusFi { is_base_in }
        | crate::types::Swap::LemmingsFi { is_base_in } => {
            swap.is_base_in = Some(*is_base_in);
        }
        crate::types::Swap::Kipseli { is_base_to_quote } => {
            swap.is_base_to_quote = Some(*is_base_to_quote);
        }
        crate::types::Swap::DynamicV2 {
            candidate_swaps,
            max_split_quote_calls,
            max_split_candidates,
        } => {
            swap.candidate_swaps_with_bps = candidate_swaps
                .iter()
                .map(ClickHouseCandidateSwapWithBps::from)
                .collect();
            swap.max_split_quote_calls = Some(*max_split_quote_calls);
            swap.max_split_candidates = Some(*max_split_candidates);
        }
        crate::types::Swap::Saber
        | crate::types::Swap::SaberAddDecimalsDeposit
        | crate::types::Swap::SaberAddDecimalsWithdraw
        | crate::types::Swap::TokenSwap
        | crate::types::Swap::Sencha
        | crate::types::Swap::Step
        | crate::types::Swap::Cropper
        | crate::types::Swap::Raydium
        | crate::types::Swap::Lifinity
        | crate::types::Swap::Mercurial
        | crate::types::Swap::Cykura
        | crate::types::Swap::MarinadeDeposit
        | crate::types::Swap::MarinadeUnstake
        | crate::types::Swap::Meteora
        | crate::types::Swap::GooseFX
        | crate::types::Swap::Balansol
        | crate::types::Swap::LifinityV2
        | crate::types::Swap::RaydiumClmm
        | crate::types::Swap::TokenSwapV2
        | crate::types::Swap::HeliumTreasuryManagementRedeemV0
        | crate::types::Swap::StakeDexStakeWrappedSol
        | crate::types::Swap::GooseFXV2
        | crate::types::Swap::Perps
        | crate::types::Swap::PerpsAddLiquidity
        | crate::types::Swap::PerpsRemoveLiquidity
        | crate::types::Swap::MeteoraDlmm
        | crate::types::Swap::RaydiumClmmV2
        | crate::types::Swap::RaydiumCP
        | crate::types::Swap::OneIntro
        | crate::types::Swap::PumpWrappedBuy
        | crate::types::Swap::PumpWrappedSell
        | crate::types::Swap::PerpsV2
        | crate::types::Swap::PerpsV2AddLiquidity
        | crate::types::Swap::PerpsV2RemoveLiquidity
        | crate::types::Swap::MoonshotWrappedBuy
        | crate::types::Swap::MoonshotWrappedSell
        | crate::types::Swap::StabbleStableSwap
        | crate::types::Swap::StabbleWeightedSwap
        | crate::types::Swap::FoxBuyFromEstimatedCost
        | crate::types::Swap::SolayerDelegateNoInit
        | crate::types::Swap::SolayerUndelegateNoInit
        | crate::types::Swap::DaosFunBuy
        | crate::types::Swap::DaosFunSell
        | crate::types::Swap::ZeroFi
        | crate::types::Swap::StakeDexWithdrawWrappedSol
        | crate::types::Swap::VirtualsBuy
        | crate::types::Swap::VirtualsSell
        | crate::types::Swap::PumpSwapBuy
        | crate::types::Swap::PumpSwapSell
        | crate::types::Swap::Gamma
        | crate::types::Swap::Woofi
        | crate::types::Swap::MeteoraDammV2
        | crate::types::Swap::MeteoraDynamicBondingCurveSwap
        | crate::types::Swap::StabbleStableSwapV2
        | crate::types::Swap::StabbleWeightedSwapV2
        | crate::types::Swap::BoopdotfunWrappedBuy
        | crate::types::Swap::BoopdotfunWrappedSell
        | crate::types::Swap::MeteoraDynamicBondingCurveSwapWithRemainingAccounts
        | crate::types::Swap::PumpWrappedBuyV2
        | crate::types::Swap::PumpWrappedSellV2
        | crate::types::Swap::PumpSwapBuyV2
        | crate::types::Swap::PumpSwapSellV2
        | crate::types::Swap::Aquifer
        | crate::types::Swap::PumpWrappedBuyV3
        | crate::types::Swap::PumpWrappedSellV3
        | crate::types::Swap::PumpSwapBuyV3
        | crate::types::Swap::PumpSwapSellV3
        | crate::types::Swap::JupiterLendDeposit
        | crate::types::Swap::JupiterLendRedeem
        | crate::types::Swap::RaydiumV2
        | crate::types::Swap::MeteoraDammV2WithRemainingAccounts
        | crate::types::Swap::Obsidian
        | crate::types::Swap::PumpWrappedBuyV4
        | crate::types::Swap::PumpWrappedSellV4
        | crate::types::Swap::CarrotIssue
        | crate::types::Swap::CarrotRedeem
        | crate::types::Swap::XOrca
        | crate::types::Swap::RunnerRodeo
        | crate::types::Swap::Omnipair
        | crate::types::Swap::MSwap
        | crate::types::Swap::VoltrDeposit
        | crate::types::Swap::VoltrWithdraw
        | crate::types::Swap::ScaleVmmBuy
        | crate::types::Swap::ScaleVmmSell
        | crate::types::Swap::ScaleAmmBuy
        | crate::types::Swap::ScaleAmmSell
        | crate::types::Swap::Trends
        | crate::types::Swap::HumaDeposit
        | crate::types::Swap::HumaInstantWithdraw => {}
    }

    swap
}

fn clickhouse_remaining_accounts_slices(
    value: &crate::types::RemainingAccountsInfo,
) -> Vec<ClickHouseRemainingAccountsSlice> {
    value
        .slices
        .iter()
        .map(ClickHouseRemainingAccountsSlice::from)
        .collect()
}

fn clickhouse_enum_variant<T>(value: &T) -> String
where
    T: serde::Serialize + std::fmt::Debug + ?Sized,
{
    match serde_json::to_value(value) {
        Ok(serde_json::Value::String(variant)) => variant,
        Ok(serde_json::Value::Object(object)) if object.len() == 1 => object
            .into_iter()
            .next()
            .map(|(variant, _)| variant)
            .unwrap_or_default(),
        _ => clickhouse_debug_variant(value),
    }
}

fn clickhouse_debug_variant<T>(value: &T) -> String
where
    T: std::fmt::Debug + ?Sized,
{
    let debug = format!("{value:?}");
    debug
        .split(|ch: char| ch == '(' || ch == '{' || ch.is_whitespace())
        .next()
        .unwrap_or_default()
        .to_string()
}

macro_rules! define_instruction_row {
    (
        $row:ident,
        $source_ty:path,
        $family_name:literal,
        $instruction_type:literal,
        $table_name:literal,
        [$(($field:ident: $field_ty:ty = $expr:expr, $column_sql:expr)),* $(,)?]
    ) => {
        #[derive(Debug, Clone, serde::Serialize)]
        pub struct $row {
            #[serde(flatten)]
            pub metadata: JupiterSwapInstructionLandingMetadata,
            $(pub $field: $field_ty,)*
        }

        impl $row {
            pub const FAMILY_NAME: &'static str = $family_name;
            pub const INSTRUCTION_TYPE: &'static str = $instruction_type;
            pub const DEFAULT_TABLE_NAME: &'static str = $table_name;

            pub fn from_parts(
                source: &$source_ty,
                metadata: &InstructionMetadata,
                context: &ClickHouseRowContext,
            ) -> Self {
                Self::from_parts_with_ingest_ts(source, metadata, context, Utc::now())
            }

            pub fn from_parts_with_ingest_ts(
                source: &$source_ty,
                metadata: &InstructionMetadata,
                context: &ClickHouseRowContext,
                ingest_ts: DateTime<Utc>,
            ) -> Self {
                let _ = source;
                Self {
                    metadata: JupiterSwapInstructionLandingMetadata::from_parts_with_ingest_ts(
                        Self::FAMILY_NAME,
                        Self::INSTRUCTION_TYPE,
                        metadata,
                        context,
                        ingest_ts,
                    ),
                    $($field: ($expr)(source),)*
                }
            }
        }

        impl ClickHouseTable for $row {
            fn table() -> &'static str {
                Self::DEFAULT_TABLE_NAME
            }

            fn columns() -> Vec<&'static str> {
                instruction_columns(&[$(stringify!($field)),*])
            }

            fn create_table_sql(table_name: &str) -> String {
                let payload_columns = vec![$(format!("{} {}", stringify!($field), $column_sql)),*];
                create_instruction_table_sql(
                    table_name,
                    &payload_columns,
                )
            }
        }

        impl ClickHouseRow for $row {
            fn table_name(&self) -> &'static str {
                Self::table()
            }

            fn partition_key(&self) -> String {
                self.metadata.partition_key()
            }
        }
    };
}

define_instruction_row!(
    JupiterSwapClaimInstructionLandingRow,
    crate::instructions::claim::Claim,
    "jupiter_swap_claim_instruction",
    "claim",
    "jupiter_swap_claim_instruction_landing",
    [(id: u8 = |source: &crate::instructions::claim::Claim| source.id, "UInt8")]
);

define_instruction_row!(
    JupiterSwapClaimTokenInstructionLandingRow,
    crate::instructions::claim_token::ClaimToken,
    "jupiter_swap_claim_token_instruction",
    "claim_token",
    "jupiter_swap_claim_token_instruction_landing",
    [(id: u8 = |source: &crate::instructions::claim_token::ClaimToken| source.id, "UInt8")]
);

define_instruction_row!(
    JupiterSwapCloseTokenInstructionLandingRow,
    crate::instructions::close_token::CloseToken,
    "jupiter_swap_close_token_instruction",
    "close_token",
    "jupiter_swap_close_token_instruction_landing",
    [
        (id: u8 = |source: &crate::instructions::close_token::CloseToken| source.id, "UInt8"),
        (burn_all: bool = |source: &crate::instructions::close_token::CloseToken| source.burn_all, "Bool"),
    ]
);

define_instruction_row!(
    JupiterSwapCloseWsolTokenAccountInstructionLandingRow,
    crate::instructions::close_wsol_token_account::CloseWsolTokenAccount,
    "jupiter_swap_close_wsol_token_account_instruction",
    "close_wsol_token_account",
    "jupiter_swap_close_wsol_token_account_instruction_landing",
    []
);

define_instruction_row!(
    JupiterSwapCreateTokenAccountInstructionLandingRow,
    crate::instructions::create_token_account::CreateTokenAccount,
    "jupiter_swap_create_token_account_instruction",
    "create_token_account",
    "jupiter_swap_create_token_account_instruction_landing",
    [(bump: u8 = |source: &crate::instructions::create_token_account::CreateTokenAccount| source.bump, "UInt8")]
);

define_instruction_row!(
    JupiterSwapCreateTokenLedgerInstructionLandingRow,
    crate::instructions::create_token_ledger::CreateTokenLedger,
    "jupiter_swap_create_token_ledger_instruction",
    "create_token_ledger",
    "jupiter_swap_create_token_ledger_instruction_landing",
    []
);

define_instruction_row!(
    JupiterSwapExactOutRouteInstructionLandingRow,
    crate::instructions::exact_out_route::ExactOutRoute,
    "jupiter_swap_exact_out_route_instruction",
    "exact_out_route",
    "jupiter_swap_exact_out_route_instruction_landing",
    [
        (route_plan: Vec<ClickHouseRoutePlanStep> = |source: &crate::instructions::exact_out_route::ExactOutRoute| clickhouse_route_plan(&source.route_plan), route_plan_step_clickhouse_type(false)),
        (out_amount: u64 = |source: &crate::instructions::exact_out_route::ExactOutRoute| source.out_amount, "UInt64"),
        (quoted_in_amount: u64 = |source: &crate::instructions::exact_out_route::ExactOutRoute| source.quoted_in_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::exact_out_route::ExactOutRoute| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u8 = |source: &crate::instructions::exact_out_route::ExactOutRoute| source.platform_fee_bps, "UInt8"),
    ]
);

define_instruction_row!(
    JupiterSwapExactOutRouteV2InstructionLandingRow,
    crate::instructions::exact_out_route_v2::ExactOutRouteV2,
    "jupiter_swap_exact_out_route_v2_instruction",
    "exact_out_route_v2",
    "jupiter_swap_exact_out_route_v2_instruction_landing",
    [
        (out_amount: u64 = |source: &crate::instructions::exact_out_route_v2::ExactOutRouteV2| source.out_amount, "UInt64"),
        (quoted_in_amount: u64 = |source: &crate::instructions::exact_out_route_v2::ExactOutRouteV2| source.quoted_in_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::exact_out_route_v2::ExactOutRouteV2| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u16 = |source: &crate::instructions::exact_out_route_v2::ExactOutRouteV2| source.platform_fee_bps, "UInt16"),
        (positive_slippage_bps: u16 = |source: &crate::instructions::exact_out_route_v2::ExactOutRouteV2| source.positive_slippage_bps, "UInt16"),
        (route_plan: Vec<ClickHouseRoutePlanStepV2> = |source: &crate::instructions::exact_out_route_v2::ExactOutRouteV2| clickhouse_route_plan_v2(&source.route_plan), route_plan_step_clickhouse_type(true)),
    ]
);

define_instruction_row!(
    JupiterSwapRouteInstructionLandingRow,
    crate::instructions::route::Route,
    "jupiter_swap_route_instruction",
    "route",
    "jupiter_swap_route_instruction_landing",
    [
        (route_plan: Vec<ClickHouseRoutePlanStep> = |source: &crate::instructions::route::Route| clickhouse_route_plan(&source.route_plan), route_plan_step_clickhouse_type(false)),
        (in_amount: u64 = |source: &crate::instructions::route::Route| source.in_amount, "UInt64"),
        (quoted_out_amount: u64 = |source: &crate::instructions::route::Route| source.quoted_out_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::route::Route| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u8 = |source: &crate::instructions::route::Route| source.platform_fee_bps, "UInt8"),
    ]
);

define_instruction_row!(
    JupiterSwapRouteV2InstructionLandingRow,
    crate::instructions::route_v2::RouteV2,
    "jupiter_swap_route_v2_instruction",
    "route_v2",
    "jupiter_swap_route_v2_instruction_landing",
    [
        (in_amount: u64 = |source: &crate::instructions::route_v2::RouteV2| source.in_amount, "UInt64"),
        (quoted_out_amount: u64 = |source: &crate::instructions::route_v2::RouteV2| source.quoted_out_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::route_v2::RouteV2| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u16 = |source: &crate::instructions::route_v2::RouteV2| source.platform_fee_bps, "UInt16"),
        (positive_slippage_bps: u16 = |source: &crate::instructions::route_v2::RouteV2| source.positive_slippage_bps, "UInt16"),
        (route_plan: Vec<ClickHouseRoutePlanStepV2> = |source: &crate::instructions::route_v2::RouteV2| clickhouse_route_plan_v2(&source.route_plan), route_plan_step_clickhouse_type(true)),
    ]
);

define_instruction_row!(
    JupiterSwapRouteWithTokenLedgerInstructionLandingRow,
    crate::instructions::route_with_token_ledger::RouteWithTokenLedger,
    "jupiter_swap_route_with_token_ledger_instruction",
    "route_with_token_ledger",
    "jupiter_swap_route_with_token_ledger_instruction_landing",
    [
        (route_plan: Vec<ClickHouseRoutePlanStep> = |source: &crate::instructions::route_with_token_ledger::RouteWithTokenLedger| clickhouse_route_plan(&source.route_plan), route_plan_step_clickhouse_type(false)),
        (quoted_out_amount: u64 = |source: &crate::instructions::route_with_token_ledger::RouteWithTokenLedger| source.quoted_out_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::route_with_token_ledger::RouteWithTokenLedger| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u8 = |source: &crate::instructions::route_with_token_ledger::RouteWithTokenLedger| source.platform_fee_bps, "UInt8"),
    ]
);

define_instruction_row!(
    JupiterSwapSetTokenLedgerInstructionLandingRow,
    crate::instructions::set_token_ledger::SetTokenLedger,
    "jupiter_swap_set_token_ledger_instruction",
    "set_token_ledger",
    "jupiter_swap_set_token_ledger_instruction_landing",
    []
);

define_instruction_row!(
    JupiterSwapSharedAccountsExactOutRouteInstructionLandingRow,
    crate::instructions::shared_accounts_exact_out_route::SharedAccountsExactOutRoute,
    "jupiter_swap_shared_accounts_exact_out_route_instruction",
    "shared_accounts_exact_out_route",
    "jupiter_swap_shared_accounts_exact_out_route_instruction_landing",
    [
        (id: u8 = |source: &crate::instructions::shared_accounts_exact_out_route::SharedAccountsExactOutRoute| source.id, "UInt8"),
        (route_plan: Vec<ClickHouseRoutePlanStep> = |source: &crate::instructions::shared_accounts_exact_out_route::SharedAccountsExactOutRoute| clickhouse_route_plan(&source.route_plan), route_plan_step_clickhouse_type(false)),
        (out_amount: u64 = |source: &crate::instructions::shared_accounts_exact_out_route::SharedAccountsExactOutRoute| source.out_amount, "UInt64"),
        (quoted_in_amount: u64 = |source: &crate::instructions::shared_accounts_exact_out_route::SharedAccountsExactOutRoute| source.quoted_in_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::shared_accounts_exact_out_route::SharedAccountsExactOutRoute| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u8 = |source: &crate::instructions::shared_accounts_exact_out_route::SharedAccountsExactOutRoute| source.platform_fee_bps, "UInt8"),
    ]
);

define_instruction_row!(
    JupiterSwapSharedAccountsExactOutRouteV2InstructionLandingRow,
    crate::instructions::shared_accounts_exact_out_route_v2::SharedAccountsExactOutRouteV2,
    "jupiter_swap_shared_accounts_exact_out_route_v2_instruction",
    "shared_accounts_exact_out_route_v2",
    "jupiter_swap_shared_accounts_exact_out_route_v2_instruction_landing",
    [
        (id: u8 = |source: &crate::instructions::shared_accounts_exact_out_route_v2::SharedAccountsExactOutRouteV2| source.id, "UInt8"),
        (out_amount: u64 = |source: &crate::instructions::shared_accounts_exact_out_route_v2::SharedAccountsExactOutRouteV2| source.out_amount, "UInt64"),
        (quoted_in_amount: u64 = |source: &crate::instructions::shared_accounts_exact_out_route_v2::SharedAccountsExactOutRouteV2| source.quoted_in_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::shared_accounts_exact_out_route_v2::SharedAccountsExactOutRouteV2| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u16 = |source: &crate::instructions::shared_accounts_exact_out_route_v2::SharedAccountsExactOutRouteV2| source.platform_fee_bps, "UInt16"),
        (positive_slippage_bps: u16 = |source: &crate::instructions::shared_accounts_exact_out_route_v2::SharedAccountsExactOutRouteV2| source.positive_slippage_bps, "UInt16"),
        (route_plan: Vec<ClickHouseRoutePlanStepV2> = |source: &crate::instructions::shared_accounts_exact_out_route_v2::SharedAccountsExactOutRouteV2| clickhouse_route_plan_v2(&source.route_plan), route_plan_step_clickhouse_type(true)),
    ]
);

define_instruction_row!(
    JupiterSwapSharedAccountsRouteInstructionLandingRow,
    crate::instructions::shared_accounts_route::SharedAccountsRoute,
    "jupiter_swap_shared_accounts_route_instruction",
    "shared_accounts_route",
    "jupiter_swap_shared_accounts_route_instruction_landing",
    [
        (id: u8 = |source: &crate::instructions::shared_accounts_route::SharedAccountsRoute| source.id, "UInt8"),
        (route_plan: Vec<ClickHouseRoutePlanStep> = |source: &crate::instructions::shared_accounts_route::SharedAccountsRoute| clickhouse_route_plan(&source.route_plan), route_plan_step_clickhouse_type(false)),
        (in_amount: u64 = |source: &crate::instructions::shared_accounts_route::SharedAccountsRoute| source.in_amount, "UInt64"),
        (quoted_out_amount: u64 = |source: &crate::instructions::shared_accounts_route::SharedAccountsRoute| source.quoted_out_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::shared_accounts_route::SharedAccountsRoute| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u8 = |source: &crate::instructions::shared_accounts_route::SharedAccountsRoute| source.platform_fee_bps, "UInt8"),
    ]
);

define_instruction_row!(
    JupiterSwapSharedAccountsRouteV2InstructionLandingRow,
    crate::instructions::shared_accounts_route_v2::SharedAccountsRouteV2,
    "jupiter_swap_shared_accounts_route_v2_instruction",
    "shared_accounts_route_v2",
    "jupiter_swap_shared_accounts_route_v2_instruction_landing",
    [
        (id: u8 = |source: &crate::instructions::shared_accounts_route_v2::SharedAccountsRouteV2| source.id, "UInt8"),
        (in_amount: u64 = |source: &crate::instructions::shared_accounts_route_v2::SharedAccountsRouteV2| source.in_amount, "UInt64"),
        (quoted_out_amount: u64 = |source: &crate::instructions::shared_accounts_route_v2::SharedAccountsRouteV2| source.quoted_out_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::shared_accounts_route_v2::SharedAccountsRouteV2| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u16 = |source: &crate::instructions::shared_accounts_route_v2::SharedAccountsRouteV2| source.platform_fee_bps, "UInt16"),
        (positive_slippage_bps: u16 = |source: &crate::instructions::shared_accounts_route_v2::SharedAccountsRouteV2| source.positive_slippage_bps, "UInt16"),
        (route_plan: Vec<ClickHouseRoutePlanStepV2> = |source: &crate::instructions::shared_accounts_route_v2::SharedAccountsRouteV2| clickhouse_route_plan_v2(&source.route_plan), route_plan_step_clickhouse_type(true)),
    ]
);

define_instruction_row!(
    JupiterSwapSharedAccountsRouteWithTokenLedgerInstructionLandingRow,
    crate::instructions::shared_accounts_route_with_token_ledger::SharedAccountsRouteWithTokenLedger,
    "jupiter_swap_shared_accounts_route_with_token_ledger_instruction",
    "shared_accounts_route_with_token_ledger",
    "jupiter_swap_shared_accounts_route_with_token_ledger_instruction_landing",
    [
        (id: u8 = |source: &crate::instructions::shared_accounts_route_with_token_ledger::SharedAccountsRouteWithTokenLedger| source.id, "UInt8"),
        (route_plan: Vec<ClickHouseRoutePlanStep> = |source: &crate::instructions::shared_accounts_route_with_token_ledger::SharedAccountsRouteWithTokenLedger| clickhouse_route_plan(&source.route_plan), route_plan_step_clickhouse_type(false)),
        (quoted_out_amount: u64 = |source: &crate::instructions::shared_accounts_route_with_token_ledger::SharedAccountsRouteWithTokenLedger| source.quoted_out_amount, "UInt64"),
        (slippage_bps: u16 = |source: &crate::instructions::shared_accounts_route_with_token_ledger::SharedAccountsRouteWithTokenLedger| source.slippage_bps, "UInt16"),
        (platform_fee_bps: u8 = |source: &crate::instructions::shared_accounts_route_with_token_ledger::SharedAccountsRouteWithTokenLedger| source.platform_fee_bps, "UInt8"),
    ]
);

#[cfg(test)]
mod tests {
    use super::*;
    use carbon_core::{clickhouse::rows::ClickHouseRows, transaction::TransactionMetadata};
    use solana_hash::Hash;
    use solana_message::VersionedMessage;
    use solana_signature::Signature;
    use solana_transaction_status::TransactionStatusMeta;
    use std::sync::Arc;

    fn metadata(block_time: Option<i64>) -> InstructionMetadata {
        let transaction_metadata = TransactionMetadata {
            slot: 55,
            signature: Signature::new_unique(),
            fee_payer: solana_pubkey::Pubkey::new_unique(),
            meta: TransactionStatusMeta::default(),
            message: VersionedMessage::Legacy(solana_message::legacy::Message::default()),
            index: Some(11),
            block_time,
            block_hash: Some(Hash::new_unique()),
        };

        InstructionMetadata {
            transaction_metadata: Arc::new(transaction_metadata),
            stack_height: 2,
            index: 3,
            absolute_path: vec![1, 2],
        }
    }

    fn context() -> ClickHouseRowContext {
        ClickHouseRowContext {
            source_name: "block_crawler".to_string(),
            mode: "backfill".to_string(),
            decoder_version: "v1".to_string(),
        }
    }

    #[test]
    fn route_instruction_row_uses_typed_columns_and_stable_id() {
        let ingest_ts = DateTime::<Utc>::from_timestamp_millis(1_704_067_200_123).unwrap();
        let metadata = metadata(Some(1_704_067_200));
        let route = crate::instructions::route::Route {
            route_plan: Vec::new(),
            in_amount: 10,
            quoted_out_amount: 20,
            slippage_bps: 30,
            platform_fee_bps: 4,
        };

        let first = JupiterSwapRouteInstructionLandingRow::from_parts_with_ingest_ts(
            &route,
            &metadata,
            &context(),
            ingest_ts,
        );
        let second = JupiterSwapRouteInstructionLandingRow::from_parts_with_ingest_ts(
            &route,
            &metadata,
            &context(),
            ingest_ts,
        );

        assert_eq!(
            first.metadata.instruction_id,
            second.metadata.instruction_id
        );
        assert_eq!(first.metadata.instruction_type, "route");
        assert_eq!(first.in_amount, 10);
        assert_eq!(first.quoted_out_amount, 20);
        assert_eq!(first.partition_key(), "2024");
        assert!(JupiterSwapRouteInstructionLandingRow::columns().contains(&"route_plan"));
    }

    #[test]
    fn wrapper_emits_normal_instruction_row() {
        let route = crate::instructions::route::Route {
            route_plan: Vec::new(),
            in_amount: 10,
            quoted_out_amount: 20,
            slippage_bps: 30,
            platform_fee_bps: 4,
        };
        let wrapper = crate::instructions::clickhouse::JupiterSwapInstructionWithClickHouseMetadata(
            crate::instructions::JupiterSwapInstruction::Route {
                program_id: crate::PROGRAM_ID,
                data: route,
                accounts: crate::instructions::route::RouteInstructionAccounts {
                    token_program: solana_pubkey::Pubkey::new_unique(),
                    user_transfer_authority: solana_pubkey::Pubkey::new_unique(),
                    user_source_token_account: solana_pubkey::Pubkey::new_unique(),
                    user_destination_token_account: solana_pubkey::Pubkey::new_unique(),
                    destination_token_account: None,
                    destination_mint: solana_pubkey::Pubkey::new_unique(),
                    platform_fee_account: None,
                    event_authority: solana_pubkey::Pubkey::new_unique(),
                    program: crate::PROGRAM_ID,
                    remaining: Vec::new(),
                },
            },
            metadata(Some(1_704_067_200)),
            Vec::new(),
        );

        let rows = wrapper.clickhouse_rows(&context());
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].table_name(),
            JupiterSwapRouteInstructionLandingRow::DEFAULT_TABLE_NAME
        );
    }
}
