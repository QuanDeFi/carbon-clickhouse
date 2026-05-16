use carbon_core::{
    clickhouse::rows::{
        deterministic_event_id, ClickHouseRow, ClickHouseRowContext, ClickHouseTable,
    },
    instruction::InstructionMetadata,
};
use chrono::{DateTime, Utc};

use crate::{
    events::{
        best_swap_out_amount_violation::BestSwapOutAmountViolationEvent,
        candidate_swap_quote_error::CandidateSwapQuoteErrorEvent,
        candidate_swap_results::CandidateSwapResultsEvent, fee_event::FeeEventEvent,
        swap_event::SwapEventEvent, swaps_event::SwapsEventEvent,
    },
    PROGRAM_ID as JUPITER_SWAP_PROGRAM_ID,
};

#[derive(Debug, Clone, serde::Serialize)]
pub struct JupiterSwapCpiEventLandingRow {
    pub program_id: String,
    pub family_name: String,
    pub event_type: String,
    pub event_id: String,
    pub slot: u64,
    pub signature: String,
    pub instruction_index: u32,
    pub stack_height: u32,
    pub absolute_path: Vec<u8>,
    pub event_seq: u32,
    pub source_name: String,
    pub mode: String,
    pub decoder_version: String,
    pub ingest_ts: String,
    pub chain_time: Option<String>,
    pub partition_time: String,
    pub block_hash: Option<String>,
    pub tx_index: Option<u64>,
    pub fee_event_account: Option<String>,
    pub fee_event_mint: Option<String>,
    pub fee_event_amount: Option<u64>,
    pub swap_event_amm: Option<String>,
    pub swap_event_input_mint: Option<String>,
    pub swap_event_input_amount: Option<u64>,
    pub swap_event_output_mint: Option<String>,
    pub swap_event_output_amount: Option<u64>,
    pub swaps_event_swap_events_present: bool,
    pub swaps_event_swap_events: Vec<ClickHouseSwapEventV2>,
    pub candidate_swap_results_results_present: bool,
    pub candidate_swap_results_results: Vec<ClickHouseCandidateSwapResult>,
    pub candidate_swap_quote_error_candidate_index: Option<u64>,
    pub candidate_swap_quote_error_in_amount: Option<u64>,
    pub candidate_swap_quote_error_error_code: Option<u64>,
    pub best_swap_out_amount_violation_expected_out_amount: Option<u64>,
    pub best_swap_out_amount_violation_out_amount: Option<u64>,
}

impl JupiterSwapCpiEventLandingRow {
    pub const FAMILY_NAME: &'static str = "jupiter_swap_cpi_event";
    pub const DEFAULT_TABLE_NAME: &'static str = "jupiter_swap_cpi_event_landing";

    pub fn from_fee_event(
        metadata: &InstructionMetadata,
        source: &FeeEventEvent,
        context: &ClickHouseRowContext,
    ) -> Self {
        let mut row = Self::base("fee_event", metadata, context, Utc::now());
        row.fee_event_account = Some(source.account.to_string());
        row.fee_event_mint = Some(source.mint.to_string());
        row.fee_event_amount = Some(source.amount);
        row
    }

    pub fn from_swap_event(
        metadata: &InstructionMetadata,
        source: &SwapEventEvent,
        context: &ClickHouseRowContext,
    ) -> Self {
        let mut row = Self::base("swap_event", metadata, context, Utc::now());
        row.swap_event_amm = Some(source.amm.to_string());
        row.swap_event_input_mint = Some(source.input_mint.to_string());
        row.swap_event_input_amount = Some(source.input_amount);
        row.swap_event_output_mint = Some(source.output_mint.to_string());
        row.swap_event_output_amount = Some(source.output_amount);
        row
    }

    pub fn from_swaps_event(
        metadata: &InstructionMetadata,
        source: &SwapsEventEvent,
        context: &ClickHouseRowContext,
    ) -> Self {
        let mut row = Self::base("swaps_event", metadata, context, Utc::now());
        row.swaps_event_swap_events_present = true;
        row.swaps_event_swap_events = clickhouse_swap_events(&source.swap_events);
        row
    }

    pub fn from_candidate_swap_results(
        metadata: &InstructionMetadata,
        source: &CandidateSwapResultsEvent,
        context: &ClickHouseRowContext,
    ) -> Self {
        let mut row = Self::base("candidate_swap_results", metadata, context, Utc::now());
        row.candidate_swap_results_results_present = true;
        row.candidate_swap_results_results = clickhouse_candidate_swap_results(&source.results);
        row
    }

    pub fn from_candidate_swap_quote_error(
        metadata: &InstructionMetadata,
        source: &CandidateSwapQuoteErrorEvent,
        context: &ClickHouseRowContext,
    ) -> Self {
        let mut row = Self::base("candidate_swap_quote_error", metadata, context, Utc::now());
        row.candidate_swap_quote_error_candidate_index = Some(source.candidate_index);
        row.candidate_swap_quote_error_in_amount = Some(source.in_amount);
        row.candidate_swap_quote_error_error_code = Some(source.error_code);
        row
    }

    pub fn from_best_swap_out_amount_violation(
        metadata: &InstructionMetadata,
        source: &BestSwapOutAmountViolationEvent,
        context: &ClickHouseRowContext,
    ) -> Self {
        let mut row = Self::base(
            "best_swap_out_amount_violation",
            metadata,
            context,
            Utc::now(),
        );
        row.best_swap_out_amount_violation_expected_out_amount = Some(source.expected_out_amount);
        row.best_swap_out_amount_violation_out_amount = Some(source.out_amount);
        row
    }

    pub fn from_swap_event_with_ingest_ts(
        metadata: &InstructionMetadata,
        source: &SwapEventEvent,
        context: &ClickHouseRowContext,
        ingest_ts: DateTime<Utc>,
    ) -> Self {
        let mut row = Self::base("swap_event", metadata, context, ingest_ts);
        row.swap_event_amm = Some(source.amm.to_string());
        row.swap_event_input_mint = Some(source.input_mint.to_string());
        row.swap_event_input_amount = Some(source.input_amount);
        row.swap_event_output_mint = Some(source.output_mint.to_string());
        row.swap_event_output_amount = Some(source.output_amount);
        row
    }

    fn base(
        event_type: &'static str,
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
        let event_seq = 0u32;

        Self {
            program_id: JUPITER_SWAP_PROGRAM_ID.to_string(),
            family_name: Self::FAMILY_NAME.to_string(),
            event_type: event_type.to_string(),
            event_id: deterministic_event_id(
                JUPITER_SWAP_PROGRAM_ID.as_ref(),
                &signature,
                &metadata.absolute_path,
                event_type,
                event_seq,
            ),
            slot: metadata.transaction_metadata.slot,
            signature,
            instruction_index: metadata.index,
            stack_height: metadata.stack_height,
            absolute_path: metadata.absolute_path.clone(),
            event_seq,
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
            fee_event_account: None,
            fee_event_mint: None,
            fee_event_amount: None,
            swap_event_amm: None,
            swap_event_input_mint: None,
            swap_event_input_amount: None,
            swap_event_output_mint: None,
            swap_event_output_amount: None,
            swaps_event_swap_events_present: false,
            swaps_event_swap_events: Vec::new(),
            candidate_swap_results_results_present: false,
            candidate_swap_results_results: Vec::new(),
            candidate_swap_quote_error_candidate_index: None,
            candidate_swap_quote_error_in_amount: None,
            candidate_swap_quote_error_error_code: None,
            best_swap_out_amount_violation_expected_out_amount: None,
            best_swap_out_amount_violation_out_amount: None,
        }
    }
}

impl ClickHouseTable for JupiterSwapCpiEventLandingRow {
    fn table() -> &'static str {
        Self::DEFAULT_TABLE_NAME
    }

    fn columns() -> Vec<&'static str> {
        vec![
            "program_id",
            "family_name",
            "event_type",
            "event_id",
            "slot",
            "signature",
            "instruction_index",
            "stack_height",
            "absolute_path",
            "event_seq",
            "source_name",
            "mode",
            "decoder_version",
            "ingest_ts",
            "chain_time",
            "partition_time",
            "block_hash",
            "tx_index",
            "fee_event_account",
            "fee_event_mint",
            "fee_event_amount",
            "swap_event_amm",
            "swap_event_input_mint",
            "swap_event_input_amount",
            "swap_event_output_mint",
            "swap_event_output_amount",
            "swaps_event_swap_events_present",
            "swaps_event_swap_events",
            "candidate_swap_results_results_present",
            "candidate_swap_results_results",
            "candidate_swap_quote_error_candidate_index",
            "candidate_swap_quote_error_in_amount",
            "candidate_swap_quote_error_error_code",
            "best_swap_out_amount_violation_expected_out_amount",
            "best_swap_out_amount_violation_out_amount",
        ]
    }

    fn create_table_sql(table_name: &str) -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (\
            program_id String,\
            family_name String,\
            event_type String,\
            event_id String,\
            slot UInt64,\
            signature String,\
            instruction_index UInt32,\
            stack_height UInt32,\
            absolute_path Array(UInt8),\
            event_seq UInt32,\
            source_name String,\
            mode String,\
            decoder_version String,\
            ingest_ts DateTime64(3, 'UTC'),\
            chain_time Nullable(DateTime64(3, 'UTC')),\
            partition_time DateTime64(3, 'UTC'),\
            block_hash Nullable(String),\
            tx_index Nullable(UInt64),\
            fee_event_account Nullable(String),\
            fee_event_mint Nullable(String),\
            fee_event_amount Nullable(UInt64),\
            swap_event_amm Nullable(String),\
            swap_event_input_mint Nullable(String),\
            swap_event_input_amount Nullable(UInt64),\
            swap_event_output_mint Nullable(String),\
            swap_event_output_amount Nullable(UInt64),\
            swaps_event_swap_events_present Bool,\
            swaps_event_swap_events Array(Tuple(input_mint String, input_amount UInt64, output_mint String, output_amount UInt64)),\
            candidate_swap_results_results_present Bool,\
            candidate_swap_results_results Array(Tuple(variant Enum8('OutAmount' = 0, 'ProgramError' = 1), value_0 Nullable(UInt64))),\
            candidate_swap_quote_error_candidate_index Nullable(UInt64),\
            candidate_swap_quote_error_in_amount Nullable(UInt64),\
            candidate_swap_quote_error_error_code Nullable(UInt64),\
            best_swap_out_amount_violation_expected_out_amount Nullable(UInt64),\
            best_swap_out_amount_violation_out_amount Nullable(UInt64)\
        ) ENGINE = MergeTree \
        PARTITION BY toYear(partition_time) \
        ORDER BY (program_id, family_name, event_id, slot)"
        )
    }
}

impl ClickHouseRow for JupiterSwapCpiEventLandingRow {
    fn table_name(&self) -> &'static str {
        Self::table()
    }

    fn partition_key(&self) -> String {
        self.partition_time[..4].to_string()
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ClickHouseSwapEventV2 {
    pub input_mint: String,
    pub input_amount: u64,
    pub output_mint: String,
    pub output_amount: u64,
}

impl From<&crate::types::SwapEventV2> for ClickHouseSwapEventV2 {
    fn from(value: &crate::types::SwapEventV2) -> Self {
        Self {
            input_mint: value.input_mint.to_string(),
            input_amount: value.input_amount,
            output_mint: value.output_mint.to_string(),
            output_amount: value.output_amount,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ClickHouseCandidateSwapResult {
    pub variant: String,
    pub value_0: Option<u64>,
}

impl ClickHouseCandidateSwapResult {
    fn new(variant: String) -> Self {
        Self {
            variant,
            value_0: None,
        }
    }
}

impl Default for ClickHouseCandidateSwapResult {
    fn default() -> Self {
        Self::new("OutAmount".to_string())
    }
}

impl From<&crate::types::CandidateSwapResult> for ClickHouseCandidateSwapResult {
    fn from(value: &crate::types::CandidateSwapResult) -> Self {
        match value {
            crate::types::CandidateSwapResult::OutAmount(value) => Self {
                variant: "OutAmount".to_string(),
                value_0: Some(*value),
            },
            crate::types::CandidateSwapResult::ProgramError(value) => Self {
                variant: "ProgramError".to_string(),
                value_0: Some(*value),
            },
        }
    }
}

fn clickhouse_swap_events(value: &[crate::types::SwapEventV2]) -> Vec<ClickHouseSwapEventV2> {
    value.iter().map(ClickHouseSwapEventV2::from).collect()
}

fn clickhouse_candidate_swap_results(
    value: &[crate::types::CandidateSwapResult],
) -> Vec<ClickHouseCandidateSwapResult> {
    value
        .iter()
        .map(ClickHouseCandidateSwapResult::from)
        .collect()
}

fn format_datetime(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use carbon_core::transaction::TransactionMetadata;
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

    fn swap_event() -> SwapEventEvent {
        SwapEventEvent {
            amm: solana_pubkey::Pubkey::new_unique(),
            input_mint: solana_pubkey::Pubkey::new_unique(),
            input_amount: 12,
            output_mint: solana_pubkey::Pubkey::new_unique(),
            output_amount: 34,
        }
    }

    #[test]
    fn event_seq_is_zero() {
        let row = JupiterSwapCpiEventLandingRow::from_swap_event_with_ingest_ts(
            &metadata(Some(1_704_067_200)),
            &swap_event(),
            &context(),
            DateTime::<Utc>::from_timestamp_millis(1_704_067_200_000).unwrap(),
        );
        assert_eq!(row.event_seq, 0);
    }

    #[test]
    fn swap_event_row_mapping_is_stable() {
        let ingest_ts = DateTime::<Utc>::from_timestamp_millis(1_704_067_200_123).unwrap();
        let metadata = metadata(Some(1_704_067_200));
        let swap_event = swap_event();
        let first = JupiterSwapCpiEventLandingRow::from_swap_event_with_ingest_ts(
            &metadata,
            &swap_event,
            &context(),
            ingest_ts,
        );
        let second = JupiterSwapCpiEventLandingRow::from_swap_event_with_ingest_ts(
            &metadata,
            &swap_event,
            &context(),
            ingest_ts,
        );

        assert_eq!(first.event_id, second.event_id);
        assert_eq!(
            first.family_name,
            JupiterSwapCpiEventLandingRow::FAMILY_NAME
        );
        assert_eq!(first.event_type, "swap_event");
        assert_eq!(first.slot, 55);
        assert_eq!(first.swap_event_input_amount, Some(12));
        assert_eq!(first.swap_event_output_amount, Some(34));
        assert!(first.fee_event_amount.is_none());
        assert_eq!(first.partition_key(), "2024");
        assert_eq!(first.ingest_ts, second.ingest_ts);
    }

    #[test]
    fn partition_time_falls_back_to_ingest_time() {
        let row = JupiterSwapCpiEventLandingRow::from_swap_event_with_ingest_ts(
            &metadata(None),
            &swap_event(),
            &context(),
            DateTime::<Utc>::from_timestamp_millis(1_704_067_200_123).unwrap(),
        );

        assert!(row.chain_time.is_none());
        assert_eq!(row.partition_key(), "2024");
    }

    #[test]
    fn cpi_event_table_uses_structured_union_columns() {
        let ddl = JupiterSwapCpiEventLandingRow::create_table_sql("jupiter_swap_cpi_event_test");

        assert!(ddl.contains("event_type String"));
        assert!(ddl.contains("fee_event_amount Nullable(UInt64)"));
        assert!(ddl.contains("swap_event_input_amount Nullable(UInt64)"));
        assert!(ddl.contains("swaps_event_swap_events_present Bool"));
        assert!(ddl.contains("swaps_event_swap_events Array(Tuple"));
        assert!(ddl.contains("variant Enum8('OutAmount' = 0, 'ProgramError' = 1)"));
        assert!(!ddl.contains(&format!("data {}", "JSON")));
    }

    #[test]
    fn candidate_swap_result_uses_structured_payload_enum_tag() {
        let result = ClickHouseCandidateSwapResult::from(
            &crate::types::CandidateSwapResult::ProgramError(6001),
        );
        assert_eq!(result.variant, "ProgramError");
        assert_eq!(result.value_0, Some(6001));
    }
}
