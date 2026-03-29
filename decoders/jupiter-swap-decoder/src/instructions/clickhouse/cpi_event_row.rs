use carbon_core::{
    clickhouse::rows::{
        deterministic_event_id, ClickHouseRow, ClickHouseRowContext, ClickHouseTable,
    },
    instruction::InstructionMetadata,
};
use chrono::{DateTime, Utc};

use crate::{events::swap_event::SwapEventEvent, PROGRAM_ID as JUPITER_SWAP_PROGRAM_ID};

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct JupiterSwapSwapEventLandingRow {
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
    pub amm: String,
    pub input_mint: String,
    pub input_amount: u64,
    pub output_mint: String,
    pub output_amount: u64,
}

impl JupiterSwapSwapEventLandingRow {
    pub const FAMILY_NAME: &'static str = "jupiter_swap_swap_event";
    pub const EVENT_TYPE: &'static str = "swap_event";
    pub const DEFAULT_TABLE_NAME: &'static str = "jupiter_swap_swap_event_landing";

    pub fn from_parts(
        metadata: &InstructionMetadata,
        swap_event: &SwapEventEvent,
        context: &ClickHouseRowContext,
    ) -> Self {
        Self::from_parts_with_ingest_ts(metadata, swap_event, context, Utc::now())
    }

    pub fn from_parts_with_ingest_ts(
        metadata: &InstructionMetadata,
        swap_event: &SwapEventEvent,
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
            event_type: Self::EVENT_TYPE.to_string(),
            event_id: deterministic_event_id(
                JUPITER_SWAP_PROGRAM_ID.as_ref(),
                &signature,
                &metadata.absolute_path,
                Self::EVENT_TYPE,
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
            amm: swap_event.amm.to_string(),
            input_mint: swap_event.input_mint.to_string(),
            input_amount: swap_event.input_amount,
            output_mint: swap_event.output_mint.to_string(),
            output_amount: swap_event.output_amount,
        }
    }
}

impl ClickHouseTable for JupiterSwapSwapEventLandingRow {
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
            "amm",
            "input_mint",
            "input_amount",
            "output_mint",
            "output_amount",
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
            amm String,\
            input_mint String,\
            input_amount UInt64,\
            output_mint String,\
            output_amount UInt64\
        ) ENGINE = MergeTree \
        PARTITION BY toYear(partition_time) \
        ORDER BY (program_id, family_name, event_id, slot)"
        )
    }
}

impl ClickHouseRow for JupiterSwapSwapEventLandingRow {
    fn partition_key(&self) -> String {
        self.partition_time[..4].to_string()
    }
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
        let row = JupiterSwapSwapEventLandingRow::from_parts_with_ingest_ts(
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
        let first = JupiterSwapSwapEventLandingRow::from_parts_with_ingest_ts(
            &metadata,
            &swap_event,
            &context(),
            ingest_ts,
        );
        let second = JupiterSwapSwapEventLandingRow::from_parts_with_ingest_ts(
            &metadata,
            &swap_event,
            &context(),
            ingest_ts,
        );

        assert_eq!(first.event_id, second.event_id);
        assert_eq!(
            first.family_name,
            JupiterSwapSwapEventLandingRow::FAMILY_NAME
        );
        assert_eq!(first.event_type, JupiterSwapSwapEventLandingRow::EVENT_TYPE);
        assert_eq!(first.slot, 55);
        assert_eq!(first.input_amount, 12);
        assert_eq!(first.output_amount, 34);
        assert_eq!(first.partition_key(), "2024");
        assert_eq!(first.ingest_ts, second.ingest_ts);
    }

    #[test]
    fn partition_time_falls_back_to_ingest_time() {
        let row = JupiterSwapSwapEventLandingRow::from_parts_with_ingest_ts(
            &metadata(None),
            &swap_event(),
            &context(),
            DateTime::<Utc>::from_timestamp_millis(1_704_067_200_123).unwrap(),
        );

        assert!(row.chain_time.is_none());
        assert_eq!(row.partition_key(), "2024");
    }
}
