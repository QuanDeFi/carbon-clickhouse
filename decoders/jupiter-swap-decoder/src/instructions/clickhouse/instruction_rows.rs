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

fn create_instruction_table_sql(table_name: &str, payload_columns: &[&str]) -> String {
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

macro_rules! define_instruction_row {
    (
        $row:ident,
        $source_ty:path,
        $family_name:literal,
        $instruction_type:literal,
        $table_name:literal,
        [$(($field:ident: $field_ty:ty = $expr:expr, $column_sql:literal)),* $(,)?]
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
                create_instruction_table_sql(
                    table_name,
                    &[$(concat!(stringify!($field), " ", $column_sql)),*],
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
        (route_plan: serde_json::Value = |source: &crate::instructions::exact_out_route::ExactOutRoute| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
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
        (route_plan: serde_json::Value = |source: &crate::instructions::exact_out_route_v2::ExactOutRouteV2| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
    ]
);

define_instruction_row!(
    JupiterSwapRouteInstructionLandingRow,
    crate::instructions::route::Route,
    "jupiter_swap_route_instruction",
    "route",
    "jupiter_swap_route_instruction_landing",
    [
        (route_plan: serde_json::Value = |source: &crate::instructions::route::Route| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
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
        (route_plan: serde_json::Value = |source: &crate::instructions::route_v2::RouteV2| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
    ]
);

define_instruction_row!(
    JupiterSwapRouteWithTokenLedgerInstructionLandingRow,
    crate::instructions::route_with_token_ledger::RouteWithTokenLedger,
    "jupiter_swap_route_with_token_ledger_instruction",
    "route_with_token_ledger",
    "jupiter_swap_route_with_token_ledger_instruction_landing",
    [
        (route_plan: serde_json::Value = |source: &crate::instructions::route_with_token_ledger::RouteWithTokenLedger| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
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
        (route_plan: serde_json::Value = |source: &crate::instructions::shared_accounts_exact_out_route::SharedAccountsExactOutRoute| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
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
        (route_plan: serde_json::Value = |source: &crate::instructions::shared_accounts_exact_out_route_v2::SharedAccountsExactOutRouteV2| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
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
        (route_plan: serde_json::Value = |source: &crate::instructions::shared_accounts_route::SharedAccountsRoute| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
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
        (route_plan: serde_json::Value = |source: &crate::instructions::shared_accounts_route_v2::SharedAccountsRouteV2| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
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
        (route_plan: serde_json::Value = |source: &crate::instructions::shared_accounts_route_with_token_ledger::SharedAccountsRouteWithTokenLedger| serde_json::to_value(&source.route_plan).expect("serialize clickhouse field"), "JSON"),
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
