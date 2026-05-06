use std::{marker::PhantomData, sync::Arc};

use chrono::{DateTime, Utc};
use solana_transaction_status::Reward;

use crate::{
    clickhouse::{
        admin::ClickHouseSchema,
        rows::{
            deterministic_account_deletion_id, deterministic_block_details_id,
            deterministic_transaction_id, ClickHouseRow, ClickHouseRowContext, ClickHouseRows,
            ClickHouseTable,
        },
        ClickHouseConfig,
    },
    datasource::{AccountDeletion, BlockDetails},
    instruction::InstructionMetadata,
    transaction::TransactionMetadata,
};

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct CarbonTransactionClickHouseRow {
    pub transaction_id: String,
    pub scope: String,
    pub slot: u64,
    pub signature: String,
    pub fee_payer: String,
    pub is_vote: bool,
    pub tx_index: Option<u64>,
    pub block_hash: Option<String>,
    pub chain_time: Option<String>,
    pub partition_time: String,
    pub fee: u64,
    pub status_success: bool,
    pub status_error: Option<String>,
    pub compute_units_consumed: Option<u64>,
    pub cost_units: Option<u64>,
    pub decoded_instruction_count: u64,
    pub source_name: String,
    pub mode: String,
    pub decoder_version: String,
    pub ingest_ts: String,
}

impl CarbonTransactionClickHouseRow {
    pub const SCOPE: &'static str = "carbon_core";
    pub const DEFAULT_TABLE_NAME: &'static str = "carbon_transaction_landing";

    pub fn from_parts(
        metadata: &TransactionMetadata,
        decoded_instruction_count: usize,
        context: &ClickHouseRowContext,
    ) -> Self {
        Self::from_parts_with_ingest_ts(metadata, decoded_instruction_count, context, Utc::now())
    }

    pub fn from_parts_with_ingest_ts(
        metadata: &TransactionMetadata,
        decoded_instruction_count: usize,
        context: &ClickHouseRowContext,
        ingest_ts: DateTime<Utc>,
    ) -> Self {
        let chain_time = metadata
            .block_time
            .and_then(DateTime::<Utc>::from_timestamp_secs);
        let partition_time = chain_time.unwrap_or(ingest_ts);
        let signature = metadata.signature.to_string();
        let (status_success, status_error) = transaction_status_parts(metadata);

        Self {
            transaction_id: deterministic_transaction_id(Self::SCOPE, &signature),
            scope: Self::SCOPE.to_string(),
            slot: metadata.slot,
            signature,
            fee_payer: metadata.fee_payer.to_string(),
            is_vote: metadata.is_vote,
            tx_index: metadata.index,
            block_hash: metadata.block_hash.map(|hash| hash.to_string()),
            chain_time: chain_time.map(format_datetime),
            partition_time: format_datetime(partition_time),
            fee: metadata.meta.fee,
            status_success,
            status_error,
            compute_units_consumed: metadata.meta.compute_units_consumed,
            cost_units: metadata.meta.cost_units,
            decoded_instruction_count: decoded_instruction_count as u64,
            source_name: context.source_name.clone(),
            mode: context.mode.clone(),
            decoder_version: context.decoder_version.clone(),
            ingest_ts: format_datetime(ingest_ts),
        }
    }
}

impl ClickHouseTable for CarbonTransactionClickHouseRow {
    fn table() -> &'static str {
        Self::DEFAULT_TABLE_NAME
    }

    fn columns() -> Vec<&'static str> {
        vec![
            "transaction_id",
            "scope",
            "slot",
            "signature",
            "fee_payer",
            "is_vote",
            "tx_index",
            "block_hash",
            "chain_time",
            "partition_time",
            "fee",
            "status_success",
            "status_error",
            "compute_units_consumed",
            "cost_units",
            "decoded_instruction_count",
            "source_name",
            "mode",
            "decoder_version",
            "ingest_ts",
        ]
    }

    fn create_table_sql(table_name: &str) -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (\
            transaction_id String,\
            scope String,\
            slot UInt64,\
            signature String,\
            fee_payer String,\
            is_vote Bool,\
            tx_index Nullable(UInt64),\
            block_hash Nullable(String),\
            chain_time Nullable(DateTime64(3, 'UTC')),\
            partition_time DateTime64(3, 'UTC'),\
            fee UInt64,\
            status_success Bool,\
            status_error Nullable(String),\
            compute_units_consumed Nullable(UInt64),\
            cost_units Nullable(UInt64),\
            decoded_instruction_count UInt64,\
            source_name String,\
            mode String,\
            decoder_version String,\
            ingest_ts DateTime64(3, 'UTC')\
        ) ENGINE = MergeTree \
        PARTITION BY toYear(partition_time) \
        ORDER BY (scope, signature, slot)"
        )
    }
}

impl ClickHouseRow for CarbonTransactionClickHouseRow {
    fn table_name(&self) -> &'static str {
        Self::table()
    }

    fn partition_key(&self) -> String {
        self.partition_time[..4].to_string()
    }
}

#[derive(Debug, Clone)]
pub struct CarbonTransactionWithClickHouseMetadata<T> {
    pub metadata: Arc<TransactionMetadata>,
    pub instructions: Vec<(InstructionMetadata, T)>,
}

impl<T> From<(Arc<TransactionMetadata>, Vec<(InstructionMetadata, T)>)>
    for CarbonTransactionWithClickHouseMetadata<T>
{
    fn from(value: (Arc<TransactionMetadata>, Vec<(InstructionMetadata, T)>)) -> Self {
        Self {
            metadata: value.0,
            instructions: value.1,
        }
    }
}

impl<T> ClickHouseRows<CarbonTransactionClickHouseRow>
    for CarbonTransactionWithClickHouseMetadata<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn clickhouse_rows(
        &self,
        context: &ClickHouseRowContext,
    ) -> Vec<CarbonTransactionClickHouseRow> {
        vec![CarbonTransactionClickHouseRow::from_parts(
            &self.metadata,
            self.instructions.len(),
            context,
        )]
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct CarbonAccountDeletionClickHouseRow {
    pub account_deletion_id: String,
    pub slot: u64,
    pub pubkey: String,
    pub transaction_signature: Option<String>,
    pub source_name: String,
    pub mode: String,
    pub decoder_version: String,
    pub ingest_ts: String,
    pub partition_slot: u64,
}

impl CarbonAccountDeletionClickHouseRow {
    pub const DEFAULT_TABLE_NAME: &'static str = "carbon_account_deletion_landing";

    pub fn from_parts(deletion: &AccountDeletion, context: &ClickHouseRowContext) -> Self {
        Self::from_parts_with_ingest_ts(deletion, context, Utc::now())
    }

    pub fn from_parts_with_ingest_ts(
        deletion: &AccountDeletion,
        context: &ClickHouseRowContext,
        ingest_ts: DateTime<Utc>,
    ) -> Self {
        let transaction_signature = deletion
            .transaction_signature
            .as_ref()
            .map(|signature| signature.to_string());

        Self {
            account_deletion_id: deterministic_account_deletion_id(
                deletion.pubkey.as_ref(),
                deletion.slot,
                transaction_signature.as_deref(),
            ),
            slot: deletion.slot,
            pubkey: deletion.pubkey.to_string(),
            transaction_signature,
            source_name: context.source_name.clone(),
            mode: context.mode.clone(),
            decoder_version: context.decoder_version.clone(),
            ingest_ts: format_datetime(ingest_ts),
            partition_slot: deletion.slot / 1_000_000,
        }
    }
}

impl ClickHouseTable for CarbonAccountDeletionClickHouseRow {
    fn table() -> &'static str {
        Self::DEFAULT_TABLE_NAME
    }

    fn columns() -> Vec<&'static str> {
        vec![
            "account_deletion_id",
            "slot",
            "pubkey",
            "transaction_signature",
            "source_name",
            "mode",
            "decoder_version",
            "ingest_ts",
            "partition_slot",
        ]
    }

    fn create_table_sql(table_name: &str) -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (\
            account_deletion_id String,\
            slot UInt64,\
            pubkey String,\
            transaction_signature Nullable(String),\
            source_name String,\
            mode String,\
            decoder_version String,\
            ingest_ts DateTime64(3, 'UTC'),\
            partition_slot UInt64\
        ) ENGINE = MergeTree \
        PARTITION BY partition_slot \
        ORDER BY (pubkey, slot, account_deletion_id)"
        )
    }
}

impl ClickHouseRow for CarbonAccountDeletionClickHouseRow {
    fn table_name(&self) -> &'static str {
        Self::table()
    }

    fn partition_key(&self) -> String {
        self.partition_slot.to_string()
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct ClickHouseBlockReward {
    pub pubkey: String,
    pub lamports: i64,
    pub post_balance: u64,
    pub reward_type: Option<String>,
    pub commission: Option<u8>,
}

impl From<&Reward> for ClickHouseBlockReward {
    fn from(value: &Reward) -> Self {
        Self {
            pubkey: value.pubkey.clone(),
            lamports: value.lamports,
            post_balance: value.post_balance,
            reward_type: value.reward_type.map(|reward_type| reward_type.to_string()),
            commission: value.commission,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, PartialEq, Eq)]
pub struct CarbonBlockDetailsClickHouseRow {
    pub block_id: String,
    pub slot: u64,
    pub block_hash: Option<String>,
    pub previous_block_hash: Option<String>,
    pub block_time: Option<String>,
    pub block_height: Option<u64>,
    pub num_reward_partitions: Option<u64>,
    pub rewards: Vec<ClickHouseBlockReward>,
    pub reward_count: u64,
    pub source_name: String,
    pub mode: String,
    pub decoder_version: String,
    pub ingest_ts: String,
    pub partition_time: String,
}

impl CarbonBlockDetailsClickHouseRow {
    pub const DEFAULT_TABLE_NAME: &'static str = "carbon_block_details_landing";

    pub fn from_parts(block_details: &BlockDetails, context: &ClickHouseRowContext) -> Self {
        Self::from_parts_with_ingest_ts(block_details, context, Utc::now())
    }

    pub fn from_parts_with_ingest_ts(
        block_details: &BlockDetails,
        context: &ClickHouseRowContext,
        ingest_ts: DateTime<Utc>,
    ) -> Self {
        let block_hash = block_details.block_hash.map(|hash| hash.to_string());
        let block_time = block_details
            .block_time
            .and_then(DateTime::<Utc>::from_timestamp_secs);
        let partition_time = block_time.unwrap_or(ingest_ts);
        let rewards: Vec<ClickHouseBlockReward> = block_details
            .rewards
            .as_ref()
            .map(|rewards| rewards.iter().map(ClickHouseBlockReward::from).collect())
            .unwrap_or_default();

        Self {
            block_id: deterministic_block_details_id(block_details.slot, block_hash.as_deref()),
            slot: block_details.slot,
            block_hash,
            previous_block_hash: block_details
                .previous_block_hash
                .map(|hash| hash.to_string()),
            block_time: block_time.map(format_datetime),
            block_height: block_details.block_height,
            num_reward_partitions: block_details.num_reward_partitions,
            reward_count: rewards.len() as u64,
            rewards,
            source_name: context.source_name.clone(),
            mode: context.mode.clone(),
            decoder_version: context.decoder_version.clone(),
            ingest_ts: format_datetime(ingest_ts),
            partition_time: format_datetime(partition_time),
        }
    }
}

impl ClickHouseTable for CarbonBlockDetailsClickHouseRow {
    fn table() -> &'static str {
        Self::DEFAULT_TABLE_NAME
    }

    fn columns() -> Vec<&'static str> {
        vec![
            "block_id",
            "slot",
            "block_hash",
            "previous_block_hash",
            "block_time",
            "block_height",
            "num_reward_partitions",
            "rewards",
            "reward_count",
            "source_name",
            "mode",
            "decoder_version",
            "ingest_ts",
            "partition_time",
        ]
    }

    fn create_table_sql(table_name: &str) -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (\
            block_id String,\
            slot UInt64,\
            block_hash Nullable(String),\
            previous_block_hash Nullable(String),\
            block_time Nullable(DateTime64(3, 'UTC')),\
            block_height Nullable(UInt64),\
            num_reward_partitions Nullable(UInt64),\
            rewards Array(Tuple(pubkey String, lamports Int64, post_balance UInt64, reward_type Nullable(String), commission Nullable(UInt8))),\
            reward_count UInt64,\
            source_name String,\
            mode String,\
            decoder_version String,\
            ingest_ts DateTime64(3, 'UTC'),\
            partition_time DateTime64(3, 'UTC')\
        ) ENGINE = MergeTree \
        PARTITION BY toYear(partition_time) \
        ORDER BY (block_id, slot)"
        )
    }
}

impl ClickHouseRow for CarbonBlockDetailsClickHouseRow {
    fn table_name(&self) -> &'static str {
        Self::table()
    }

    fn partition_key(&self) -> String {
        self.partition_time[..4].to_string()
    }
}

pub type ClickHouseCoreTransactionProcessor<T> =
    crate::clickhouse::processors::ClickHouseTransactionProcessor<
        T,
        CarbonTransactionWithClickHouseMetadata<T>,
        CarbonTransactionClickHouseRow,
    >;

pub struct ClickHouseCoreSurfacesMigration {
    _phantom: PhantomData<()>,
}

impl ClickHouseSchema for ClickHouseCoreSurfacesMigration {
    fn operations(_config: &ClickHouseConfig) -> Vec<String> {
        vec![
            CarbonTransactionClickHouseRow::create_table_sql(
                CarbonTransactionClickHouseRow::DEFAULT_TABLE_NAME,
            ),
            CarbonAccountDeletionClickHouseRow::create_table_sql(
                CarbonAccountDeletionClickHouseRow::DEFAULT_TABLE_NAME,
            ),
            CarbonBlockDetailsClickHouseRow::create_table_sql(
                CarbonBlockDetailsClickHouseRow::DEFAULT_TABLE_NAME,
            ),
        ]
    }
}

fn transaction_status_parts(metadata: &TransactionMetadata) -> (bool, Option<String>) {
    match &metadata.meta.status {
        Ok(()) => (true, None),
        Err(error) => (false, Some(format!("{error:?}"))),
    }
}

fn format_datetime(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_hash::Hash;
    use solana_message::VersionedMessage;
    use solana_signature::Signature;
    use solana_transaction_status::TransactionStatusMeta;

    fn context() -> ClickHouseRowContext {
        ClickHouseRowContext {
            source_name: "test_source".to_string(),
            mode: "backfill".to_string(),
            decoder_version: "v1".to_string(),
        }
    }

    #[test]
    fn transaction_row_preserves_vote_and_status_metadata() {
        let metadata = TransactionMetadata {
            slot: 42,
            signature: Signature::new_unique(),
            fee_payer: solana_pubkey::Pubkey::new_unique(),
            meta: TransactionStatusMeta {
                fee: 5000,
                compute_units_consumed: Some(100),
                cost_units: Some(200),
                ..Default::default()
            },
            message: VersionedMessage::Legacy(solana_message::legacy::Message::default()),
            index: Some(7),
            block_time: Some(1_704_067_200),
            block_hash: Some(Hash::new_unique()),
            is_vote: true,
        };

        let row = CarbonTransactionClickHouseRow::from_parts(&metadata, 3, &context());

        assert!(row.is_vote);
        assert_eq!(row.fee, 5000);
        assert_eq!(row.compute_units_consumed, Some(100));
        assert_eq!(row.cost_units, Some(200));
        assert_eq!(row.decoded_instruction_count, 3);
        assert_eq!(row.partition_key(), "2024");
    }

    #[test]
    fn account_deletion_row_uses_slot_partition() {
        let deletion = AccountDeletion {
            pubkey: solana_pubkey::Pubkey::new_unique(),
            slot: 2_000_001,
            transaction_signature: Some(Signature::new_unique()),
        };

        let row = CarbonAccountDeletionClickHouseRow::from_parts(&deletion, &context());

        assert_eq!(row.partition_slot, 2);
        assert_eq!(row.partition_key(), "2");
        assert!(row.transaction_signature.is_some());
    }

    #[test]
    fn block_details_ddl_uses_structured_rewards_without_json() {
        let sql = CarbonBlockDetailsClickHouseRow::create_table_sql(
            CarbonBlockDetailsClickHouseRow::DEFAULT_TABLE_NAME,
        );

        assert!(sql.contains("rewards Array(Tuple("));
        assert!(!sql.contains("JSON"));
    }

    #[test]
    fn core_surfaces_migration_creates_all_tables() {
        let config = ClickHouseConfig::new(
            "http://localhost:8123".to_string(),
            "default".to_string(),
            None,
            None,
            CarbonTransactionClickHouseRow::DEFAULT_TABLE_NAME.to_string(),
            "source".to_string(),
            "live".to_string(),
            "v1".to_string(),
            100,
            std::time::Duration::from_secs(60),
        );

        let operations = ClickHouseCoreSurfacesMigration::operations(&config);

        assert_eq!(operations.len(), 3);
        assert!(operations
            .iter()
            .any(|sql| sql.contains(CarbonTransactionClickHouseRow::DEFAULT_TABLE_NAME)));
        assert!(operations
            .iter()
            .any(|sql| sql.contains(CarbonAccountDeletionClickHouseRow::DEFAULT_TABLE_NAME)));
        assert!(operations
            .iter()
            .any(|sql| sql.contains(CarbonBlockDetailsClickHouseRow::DEFAULT_TABLE_NAME)));
    }
}
