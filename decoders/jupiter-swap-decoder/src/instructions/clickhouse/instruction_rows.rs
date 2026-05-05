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
            variant {swap_variant_type},\
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
        hylo_swap_type = hylo_swap_type_clickhouse_type(),
        swap_variant_type = swap_variant_clickhouse_type()
    )
}

fn candidate_swap_clickhouse_type() -> String {
    format!(
        "Tuple(\
        variant {},\
        side Nullable(Enum8('Bid' = 0, 'Ask' = 1)),\
        swap_id Nullable(UInt64),\
        is_base_to_quote Nullable(Bool),\
        a_to_b Nullable(Bool),\
        is_bid Nullable(Bool))",
        candidate_swap_variant_clickhouse_type()
    )
}

fn swap_variant_clickhouse_type() -> &'static str {
    "Enum16(\
        'Saber' = 0,\
        'SaberAddDecimalsDeposit' = 1,\
        'SaberAddDecimalsWithdraw' = 2,\
        'TokenSwap' = 3,\
        'Sencha' = 4,\
        'Step' = 5,\
        'Cropper' = 6,\
        'Raydium' = 7,\
        'Crema' = 8,\
        'Lifinity' = 9,\
        'Mercurial' = 10,\
        'Cykura' = 11,\
        'Serum' = 12,\
        'MarinadeDeposit' = 13,\
        'MarinadeUnstake' = 14,\
        'Aldrin' = 15,\
        'AldrinV2' = 16,\
        'Whirlpool' = 17,\
        'Invariant' = 18,\
        'Meteora' = 19,\
        'GooseFX' = 20,\
        'DeltaFi' = 21,\
        'Balansol' = 22,\
        'MarcoPolo' = 23,\
        'Dradex' = 24,\
        'LifinityV2' = 25,\
        'RaydiumClmm' = 26,\
        'Openbook' = 27,\
        'Phoenix' = 28,\
        'Symmetry' = 29,\
        'TokenSwapV2' = 30,\
        'HeliumTreasuryManagementRedeemV0' = 31,\
        'StakeDexStakeWrappedSol' = 32,\
        'StakeDexSwapViaStake' = 33,\
        'GooseFXV2' = 34,\
        'Perps' = 35,\
        'PerpsAddLiquidity' = 36,\
        'PerpsRemoveLiquidity' = 37,\
        'MeteoraDlmm' = 38,\
        'OpenBookV2' = 39,\
        'RaydiumClmmV2' = 40,\
        'StakeDexPrefundWithdrawStakeAndDepositStake' = 41,\
        'Clone' = 42,\
        'SanctumS' = 43,\
        'SanctumSAddLiquidity' = 44,\
        'SanctumSRemoveLiquidity' = 45,\
        'RaydiumCP' = 46,\
        'WhirlpoolSwapV2' = 47,\
        'OneIntro' = 48,\
        'PumpWrappedBuy' = 49,\
        'PumpWrappedSell' = 50,\
        'PerpsV2' = 51,\
        'PerpsV2AddLiquidity' = 52,\
        'PerpsV2RemoveLiquidity' = 53,\
        'MoonshotWrappedBuy' = 54,\
        'MoonshotWrappedSell' = 55,\
        'StabbleStableSwap' = 56,\
        'StabbleWeightedSwap' = 57,\
        'Obric' = 58,\
        'FoxBuyFromEstimatedCost' = 59,\
        'FoxClaimPartial' = 60,\
        'SolFi' = 61,\
        'SolayerDelegateNoInit' = 62,\
        'SolayerUndelegateNoInit' = 63,\
        'TokenMill' = 64,\
        'DaosFunBuy' = 65,\
        'DaosFunSell' = 66,\
        'ZeroFi' = 67,\
        'StakeDexWithdrawWrappedSol' = 68,\
        'VirtualsBuy' = 69,\
        'VirtualsSell' = 70,\
        'Perena' = 71,\
        'PumpSwapBuy' = 72,\
        'PumpSwapSell' = 73,\
        'Gamma' = 74,\
        'MeteoraDlmmSwapV2' = 75,\
        'Woofi' = 76,\
        'MeteoraDammV2' = 77,\
        'MeteoraDynamicBondingCurveSwap' = 78,\
        'StabbleStableSwapV2' = 79,\
        'StabbleWeightedSwapV2' = 80,\
        'RaydiumLaunchlabBuy' = 81,\
        'RaydiumLaunchlabSell' = 82,\
        'BoopdotfunWrappedBuy' = 83,\
        'BoopdotfunWrappedSell' = 84,\
        'Plasma' = 85,\
        'GoonFi' = 86,\
        'HumidiFi' = 87,\
        'MeteoraDynamicBondingCurveSwapWithRemainingAccounts' = 88,\
        'TesseraV' = 89,\
        'PumpWrappedBuyV2' = 90,\
        'PumpWrappedSellV2' = 91,\
        'PumpSwapBuyV2' = 92,\
        'PumpSwapSellV2' = 93,\
        'Heaven' = 94,\
        'SolFiV2' = 95,\
        'Aquifer' = 96,\
        'PumpWrappedBuyV3' = 97,\
        'PumpWrappedSellV3' = 98,\
        'PumpSwapBuyV3' = 99,\
        'PumpSwapSellV3' = 100,\
        'JupiterLendDeposit' = 101,\
        'JupiterLendRedeem' = 102,\
        'DefiTuna' = 103,\
        'AlphaQ' = 104,\
        'RaydiumV2' = 105,\
        'SarosDlmm' = 106,\
        'Futarchy' = 107,\
        'MeteoraDammV2WithRemainingAccounts' = 108,\
        'Obsidian' = 109,\
        'WhaleStreet' = 110,\
        'DynamicV1' = 111,\
        'PumpWrappedBuyV4' = 112,\
        'PumpWrappedSellV4' = 113,\
        'CarrotIssue' = 114,\
        'CarrotRedeem' = 115,\
        'Manifest' = 116,\
        'BisonFi' = 117,\
        'HumidiFiV2' = 118,\
        'PerenaStar' = 119,\
        'JupiterRfqV2' = 120,\
        'GoonFiV2' = 121,\
        'Scorch' = 122,\
        'VaultLiquidUnstake' = 123,\
        'XOrca' = 124,\
        'Quantum' = 125,\
        'WhaleStreetV2' = 126,\
        'Riptide' = 127,\
        'RunnerRodeo' = 128,\
        'TaurusFi' = 129,\
        'Omnipair' = 130,\
        'MSwap' = 131,\
        'Hylo' = 132,\
        'VoltrDeposit' = 133,\
        'VoltrWithdraw' = 134,\
        'SanctumSV2' = 135,\
        'LemmingsFi' = 136,\
        'ScaleVmmBuy' = 137,\
        'ScaleVmmSell' = 138,\
        'ScaleAmmBuy' = 139,\
        'ScaleAmmSell' = 140,\
        'BisonFiV2' = 141,\
        'Trends' = 142,\
        'HumaDeposit' = 143,\
        'HumaInstantWithdraw' = 144,\
        'Kipseli' = 145,\
        'DynamicV2' = 146)"
}

fn candidate_swap_variant_clickhouse_type() -> &'static str {
    "Enum8(\
        'HumidiFi' = 0,\
        'TesseraV' = 1,\
        'HumidiFiV2' = 2,\
        'RaydiumV2' = 3,\
        'RaydiumClmm' = 4,\
        'Whirlpool' = 5,\
        'ZeroFi' = 6,\
        'BisonFiV2' = 7,\
        'GoonFiV2' = 8)"
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

impl ClickHouseCandidateSwap {
    fn new(variant: String) -> Self {
        Self {
            variant,
            side: None,
            swap_id: None,
            is_base_to_quote: None,
            a_to_b: None,
            is_bid: None,
        }
    }
}

impl Default for ClickHouseCandidateSwap {
    fn default() -> Self {
        Self::new("HumidiFi".to_string())
    }
}

fn candidate_swap_variant_name(value: &crate::types::CandidateSwap) -> &'static str {
    match value {
        crate::types::CandidateSwap::HumidiFi { .. } => "HumidiFi",
        crate::types::CandidateSwap::TesseraV { .. } => "TesseraV",
        crate::types::CandidateSwap::HumidiFiV2 { .. } => "HumidiFiV2",
        crate::types::CandidateSwap::RaydiumV2 => "RaydiumV2",
        crate::types::CandidateSwap::RaydiumClmm => "RaydiumClmm",
        crate::types::CandidateSwap::Whirlpool { .. } => "Whirlpool",
        crate::types::CandidateSwap::ZeroFi => "ZeroFi",
        crate::types::CandidateSwap::BisonFiV2 { .. } => "BisonFiV2",
        crate::types::CandidateSwap::GoonFiV2 { .. } => "GoonFiV2",
    }
}

impl From<&crate::types::CandidateSwap> for ClickHouseCandidateSwap {
    fn from(value: &crate::types::CandidateSwap) -> Self {
        let mut candidate = Self::new(candidate_swap_variant_name(value).to_string());

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

impl Default for ClickHouseSwap {
    fn default() -> Self {
        Self::new("Saber".to_string())
    }
}

fn swap_variant_name(value: &crate::types::Swap) -> &'static str {
    match value {
        crate::types::Swap::Saber => "Saber",
        crate::types::Swap::SaberAddDecimalsDeposit => "SaberAddDecimalsDeposit",
        crate::types::Swap::SaberAddDecimalsWithdraw => "SaberAddDecimalsWithdraw",
        crate::types::Swap::TokenSwap => "TokenSwap",
        crate::types::Swap::Sencha => "Sencha",
        crate::types::Swap::Step => "Step",
        crate::types::Swap::Cropper => "Cropper",
        crate::types::Swap::Raydium => "Raydium",
        crate::types::Swap::Crema { .. } => "Crema",
        crate::types::Swap::Lifinity => "Lifinity",
        crate::types::Swap::Mercurial => "Mercurial",
        crate::types::Swap::Cykura => "Cykura",
        crate::types::Swap::Serum { .. } => "Serum",
        crate::types::Swap::MarinadeDeposit => "MarinadeDeposit",
        crate::types::Swap::MarinadeUnstake => "MarinadeUnstake",
        crate::types::Swap::Aldrin { .. } => "Aldrin",
        crate::types::Swap::AldrinV2 { .. } => "AldrinV2",
        crate::types::Swap::Whirlpool { .. } => "Whirlpool",
        crate::types::Swap::Invariant { .. } => "Invariant",
        crate::types::Swap::Meteora => "Meteora",
        crate::types::Swap::GooseFX => "GooseFX",
        crate::types::Swap::DeltaFi { .. } => "DeltaFi",
        crate::types::Swap::Balansol => "Balansol",
        crate::types::Swap::MarcoPolo { .. } => "MarcoPolo",
        crate::types::Swap::Dradex { .. } => "Dradex",
        crate::types::Swap::LifinityV2 => "LifinityV2",
        crate::types::Swap::RaydiumClmm => "RaydiumClmm",
        crate::types::Swap::Openbook { .. } => "Openbook",
        crate::types::Swap::Phoenix { .. } => "Phoenix",
        crate::types::Swap::Symmetry { .. } => "Symmetry",
        crate::types::Swap::TokenSwapV2 => "TokenSwapV2",
        crate::types::Swap::HeliumTreasuryManagementRedeemV0 => "HeliumTreasuryManagementRedeemV0",
        crate::types::Swap::StakeDexStakeWrappedSol => "StakeDexStakeWrappedSol",
        crate::types::Swap::StakeDexSwapViaStake { .. } => "StakeDexSwapViaStake",
        crate::types::Swap::GooseFXV2 => "GooseFXV2",
        crate::types::Swap::Perps => "Perps",
        crate::types::Swap::PerpsAddLiquidity => "PerpsAddLiquidity",
        crate::types::Swap::PerpsRemoveLiquidity => "PerpsRemoveLiquidity",
        crate::types::Swap::MeteoraDlmm => "MeteoraDlmm",
        crate::types::Swap::OpenBookV2 { .. } => "OpenBookV2",
        crate::types::Swap::RaydiumClmmV2 => "RaydiumClmmV2",
        crate::types::Swap::StakeDexPrefundWithdrawStakeAndDepositStake { .. } => {
            "StakeDexPrefundWithdrawStakeAndDepositStake"
        }
        crate::types::Swap::Clone { .. } => "Clone",
        crate::types::Swap::SanctumS { .. } => "SanctumS",
        crate::types::Swap::SanctumSAddLiquidity { .. } => "SanctumSAddLiquidity",
        crate::types::Swap::SanctumSRemoveLiquidity { .. } => "SanctumSRemoveLiquidity",
        crate::types::Swap::RaydiumCP => "RaydiumCP",
        crate::types::Swap::WhirlpoolSwapV2 { .. } => "WhirlpoolSwapV2",
        crate::types::Swap::OneIntro => "OneIntro",
        crate::types::Swap::PumpWrappedBuy => "PumpWrappedBuy",
        crate::types::Swap::PumpWrappedSell => "PumpWrappedSell",
        crate::types::Swap::PerpsV2 => "PerpsV2",
        crate::types::Swap::PerpsV2AddLiquidity => "PerpsV2AddLiquidity",
        crate::types::Swap::PerpsV2RemoveLiquidity => "PerpsV2RemoveLiquidity",
        crate::types::Swap::MoonshotWrappedBuy => "MoonshotWrappedBuy",
        crate::types::Swap::MoonshotWrappedSell => "MoonshotWrappedSell",
        crate::types::Swap::StabbleStableSwap => "StabbleStableSwap",
        crate::types::Swap::StabbleWeightedSwap => "StabbleWeightedSwap",
        crate::types::Swap::Obric { .. } => "Obric",
        crate::types::Swap::FoxBuyFromEstimatedCost => "FoxBuyFromEstimatedCost",
        crate::types::Swap::FoxClaimPartial { .. } => "FoxClaimPartial",
        crate::types::Swap::SolFi { .. } => "SolFi",
        crate::types::Swap::SolayerDelegateNoInit => "SolayerDelegateNoInit",
        crate::types::Swap::SolayerUndelegateNoInit => "SolayerUndelegateNoInit",
        crate::types::Swap::TokenMill { .. } => "TokenMill",
        crate::types::Swap::DaosFunBuy => "DaosFunBuy",
        crate::types::Swap::DaosFunSell => "DaosFunSell",
        crate::types::Swap::ZeroFi => "ZeroFi",
        crate::types::Swap::StakeDexWithdrawWrappedSol => "StakeDexWithdrawWrappedSol",
        crate::types::Swap::VirtualsBuy => "VirtualsBuy",
        crate::types::Swap::VirtualsSell => "VirtualsSell",
        crate::types::Swap::Perena { .. } => "Perena",
        crate::types::Swap::PumpSwapBuy => "PumpSwapBuy",
        crate::types::Swap::PumpSwapSell => "PumpSwapSell",
        crate::types::Swap::Gamma => "Gamma",
        crate::types::Swap::MeteoraDlmmSwapV2 { .. } => "MeteoraDlmmSwapV2",
        crate::types::Swap::Woofi => "Woofi",
        crate::types::Swap::MeteoraDammV2 => "MeteoraDammV2",
        crate::types::Swap::MeteoraDynamicBondingCurveSwap => "MeteoraDynamicBondingCurveSwap",
        crate::types::Swap::StabbleStableSwapV2 => "StabbleStableSwapV2",
        crate::types::Swap::StabbleWeightedSwapV2 => "StabbleWeightedSwapV2",
        crate::types::Swap::RaydiumLaunchlabBuy { .. } => "RaydiumLaunchlabBuy",
        crate::types::Swap::RaydiumLaunchlabSell { .. } => "RaydiumLaunchlabSell",
        crate::types::Swap::BoopdotfunWrappedBuy => "BoopdotfunWrappedBuy",
        crate::types::Swap::BoopdotfunWrappedSell => "BoopdotfunWrappedSell",
        crate::types::Swap::Plasma { .. } => "Plasma",
        crate::types::Swap::GoonFi { .. } => "GoonFi",
        crate::types::Swap::HumidiFi { .. } => "HumidiFi",
        crate::types::Swap::MeteoraDynamicBondingCurveSwapWithRemainingAccounts => {
            "MeteoraDynamicBondingCurveSwapWithRemainingAccounts"
        }
        crate::types::Swap::TesseraV { .. } => "TesseraV",
        crate::types::Swap::PumpWrappedBuyV2 => "PumpWrappedBuyV2",
        crate::types::Swap::PumpWrappedSellV2 => "PumpWrappedSellV2",
        crate::types::Swap::PumpSwapBuyV2 => "PumpSwapBuyV2",
        crate::types::Swap::PumpSwapSellV2 => "PumpSwapSellV2",
        crate::types::Swap::Heaven { .. } => "Heaven",
        crate::types::Swap::SolFiV2 { .. } => "SolFiV2",
        crate::types::Swap::Aquifer => "Aquifer",
        crate::types::Swap::PumpWrappedBuyV3 => "PumpWrappedBuyV3",
        crate::types::Swap::PumpWrappedSellV3 => "PumpWrappedSellV3",
        crate::types::Swap::PumpSwapBuyV3 => "PumpSwapBuyV3",
        crate::types::Swap::PumpSwapSellV3 => "PumpSwapSellV3",
        crate::types::Swap::JupiterLendDeposit => "JupiterLendDeposit",
        crate::types::Swap::JupiterLendRedeem => "JupiterLendRedeem",
        crate::types::Swap::DefiTuna { .. } => "DefiTuna",
        crate::types::Swap::AlphaQ { .. } => "AlphaQ",
        crate::types::Swap::RaydiumV2 => "RaydiumV2",
        crate::types::Swap::SarosDlmm { .. } => "SarosDlmm",
        crate::types::Swap::Futarchy { .. } => "Futarchy",
        crate::types::Swap::MeteoraDammV2WithRemainingAccounts => {
            "MeteoraDammV2WithRemainingAccounts"
        }
        crate::types::Swap::Obsidian => "Obsidian",
        crate::types::Swap::WhaleStreet { .. } => "WhaleStreet",
        crate::types::Swap::DynamicV1 { .. } => "DynamicV1",
        crate::types::Swap::PumpWrappedBuyV4 => "PumpWrappedBuyV4",
        crate::types::Swap::PumpWrappedSellV4 => "PumpWrappedSellV4",
        crate::types::Swap::CarrotIssue => "CarrotIssue",
        crate::types::Swap::CarrotRedeem => "CarrotRedeem",
        crate::types::Swap::Manifest { .. } => "Manifest",
        crate::types::Swap::BisonFi { .. } => "BisonFi",
        crate::types::Swap::HumidiFiV2 { .. } => "HumidiFiV2",
        crate::types::Swap::PerenaStar { .. } => "PerenaStar",
        crate::types::Swap::JupiterRfqV2 { .. } => "JupiterRfqV2",
        crate::types::Swap::GoonFiV2 { .. } => "GoonFiV2",
        crate::types::Swap::Scorch { .. } => "Scorch",
        crate::types::Swap::VaultLiquidUnstake { .. } => "VaultLiquidUnstake",
        crate::types::Swap::XOrca => "XOrca",
        crate::types::Swap::Quantum { .. } => "Quantum",
        crate::types::Swap::WhaleStreetV2 { .. } => "WhaleStreetV2",
        crate::types::Swap::Riptide { .. } => "Riptide",
        crate::types::Swap::RunnerRodeo => "RunnerRodeo",
        crate::types::Swap::TaurusFi { .. } => "TaurusFi",
        crate::types::Swap::Omnipair => "Omnipair",
        crate::types::Swap::MSwap => "MSwap",
        crate::types::Swap::Hylo { .. } => "Hylo",
        crate::types::Swap::VoltrDeposit => "VoltrDeposit",
        crate::types::Swap::VoltrWithdraw => "VoltrWithdraw",
        crate::types::Swap::SanctumSV2 { .. } => "SanctumSV2",
        crate::types::Swap::LemmingsFi { .. } => "LemmingsFi",
        crate::types::Swap::ScaleVmmBuy => "ScaleVmmBuy",
        crate::types::Swap::ScaleVmmSell => "ScaleVmmSell",
        crate::types::Swap::ScaleAmmBuy => "ScaleAmmBuy",
        crate::types::Swap::ScaleAmmSell => "ScaleAmmSell",
        crate::types::Swap::BisonFiV2 { .. } => "BisonFiV2",
        crate::types::Swap::Trends => "Trends",
        crate::types::Swap::HumaDeposit => "HumaDeposit",
        crate::types::Swap::HumaInstantWithdraw => "HumaInstantWithdraw",
        crate::types::Swap::Kipseli { .. } => "Kipseli",
        crate::types::Swap::DynamicV2 { .. } => "DynamicV2",
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
            swap: ClickHouseSwap::from(&value.swap),
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
            swap: ClickHouseSwap::from(&value.swap),
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

impl From<&crate::types::Swap> for ClickHouseSwap {
    fn from(value: &crate::types::Swap) -> Self {
        let mut swap = Self::new(swap_variant_name(value).to_string());

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
            | crate::types::Swap::StakeDexPrefundWithdrawStakeAndDepositStake {
                bridge_stake_seed,
            } => {
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
    fn route_plan_uses_structured_payload_enum_tags() {
        let ddl =
            JupiterSwapRouteInstructionLandingRow::create_table_sql("jupiter_swap_route_test");

        assert!(ddl.contains("variant Enum16("));
        assert!(ddl.contains("candidate_swaps Array(Tuple(variant Enum8("));
        assert!(!ddl.contains("variant LowCardinality(String)"));

        let step = crate::types::RoutePlanStep {
            swap: crate::types::Swap::HumidiFi {
                swap_id: 42,
                is_base_to_quote: true,
            },
            percent: 100,
            input_index: 0,
            output_index: 1,
        };
        let row_step = ClickHouseRoutePlanStep::from(&step);

        assert_eq!(row_step.swap.variant, "HumidiFi");
        assert_eq!(
            row_step.swap.swap_id.as_ref().map(|value| value.0),
            Some(42)
        );
        assert_eq!(row_step.swap.is_base_to_quote, Some(true));
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
