use carbon_core::{
    account::{AccountMetadata, DecodedAccount},
    clickhouse::rows::{
        deterministic_account_id, ClickHouseRow, ClickHouseRowContext, ClickHouseTable,
    },
};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize)]
pub struct TokenAccountClickHouseRow {
    pub program_id: String,
    pub family_name: String,
    pub account_type: String,
    pub account_id: String,
    pub slot: u64,
    pub pubkey: String,
    pub transaction_signature: Option<String>,
    pub lamports: u64,
    pub owner: String,
    pub executable: bool,
    pub rent_epoch: u64,
    pub source_name: String,
    pub mode: String,
    pub decoder_version: String,
    pub ingest_ts: String,
    pub partition_slot: u64,
    pub mint: String,
    pub token_owner: String,
    pub amount: u64,
    pub delegate: Option<String>,
    pub state: String,
    pub is_native: Option<u64>,
    pub delegated_amount: u64,
    pub close_authority: Option<String>,
}

impl TokenAccountClickHouseRow {
    pub const FAMILY_NAME: &'static str = "token_program_token_account";
    pub const ACCOUNT_TYPE: &'static str = "token";
    pub const DEFAULT_TABLE_NAME: &'static str = "token_program_token_account_landing";

    pub fn from_parts(
        source: crate::accounts::token::Token,
        decoded_account: &DecodedAccount<crate::accounts::TokenProgramAccount>,
        metadata: &AccountMetadata,
        context: &ClickHouseRowContext,
    ) -> Self {
        let ingest_ts = Utc::now();
        let partition_slot = metadata.slot / 1_000_000;

        Self {
            program_id: crate::PROGRAM_ID.to_string(),
            family_name: Self::FAMILY_NAME.to_string(),
            account_type: Self::ACCOUNT_TYPE.to_string(),
            account_id: deterministic_account_id(
                crate::PROGRAM_ID.as_ref(),
                metadata.pubkey.as_ref(),
                metadata.slot,
                Self::ACCOUNT_TYPE,
            ),
            slot: metadata.slot,
            pubkey: metadata.pubkey.to_string(),
            transaction_signature: metadata
                .transaction_signature
                .as_ref()
                .map(|signature| signature.to_string()),
            lamports: decoded_account.lamports,
            owner: decoded_account.owner.to_string(),
            executable: decoded_account.executable,
            rent_epoch: decoded_account.rent_epoch,
            source_name: context.source_name.clone(),
            mode: context.mode.clone(),
            decoder_version: context.decoder_version.clone(),
            ingest_ts: format_datetime(ingest_ts),
            partition_slot,
            mint: source.mint.to_string(),
            token_owner: source.owner.to_string(),
            amount: source.amount,
            delegate: source.delegate.map(|value| value.to_string()),
            state: clickhouse_enum_variant(&source.state),
            is_native: source.is_native,
            delegated_amount: source.delegated_amount,
            close_authority: source.close_authority.map(|value| value.to_string()),
        }
    }
}

impl ClickHouseTable for TokenAccountClickHouseRow {
    fn table() -> &'static str {
        Self::DEFAULT_TABLE_NAME
    }

    fn columns() -> Vec<&'static str> {
        vec![
            "program_id",
            "family_name",
            "account_type",
            "account_id",
            "slot",
            "pubkey",
            "transaction_signature",
            "lamports",
            "owner",
            "executable",
            "rent_epoch",
            "source_name",
            "mode",
            "decoder_version",
            "ingest_ts",
            "partition_slot",
            "mint",
            "token_owner",
            "amount",
            "delegate",
            "state",
            "is_native",
            "delegated_amount",
            "close_authority",
        ]
    }

    fn create_table_sql(table_name: &str) -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (\
            program_id String,\
            family_name String,\
            account_type String,\
            account_id String,\
            slot UInt64,\
            pubkey String,\
            transaction_signature Nullable(String),\
            lamports UInt64,\
            owner String,\
            executable Bool,\
            rent_epoch UInt64,\
            source_name String,\
            mode String,\
            decoder_version String,\
            ingest_ts DateTime64(3, 'UTC'),\
            partition_slot UInt64,\
            mint String,\
            token_owner String,\
            amount UInt64,\
            delegate Nullable(String),\
            state Enum8('Uninitialized' = 0, 'Initialized' = 1, 'Frozen' = 2),\
            is_native Nullable(UInt64),\
            delegated_amount UInt64,\
            close_authority Nullable(String)\
        ) ENGINE = MergeTree \
        PARTITION BY partition_slot \
        ORDER BY (program_id, family_name, account_id, slot)"
        )
    }
}

impl TokenAccountClickHouseRow {
    pub fn migration_operations(table_name: &str) -> Vec<String> {
        let mut operations = Vec::new();
        operations.push(Self::create_table_sql(table_name));
        operations.extend(Self::add_column_sql(table_name));
        operations
    }

    pub fn add_column_sql(table_name: &str) -> Vec<String> {
        vec![
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS program_id String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS family_name String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS account_type String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS account_id String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS slot UInt64"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS pubkey String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS transaction_signature Nullable(String)"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS lamports UInt64"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS owner String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS executable Bool"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS rent_epoch UInt64"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS source_name String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS mode String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS decoder_version String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS ingest_ts DateTime64(3, 'UTC')"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS partition_slot UInt64"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS mint String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS token_owner String"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS amount UInt64"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS delegate Nullable(String)"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS state Enum8('Uninitialized' = 0, 'Initialized' = 1, 'Frozen' = 2)"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS is_native Nullable(UInt64)"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS delegated_amount UInt64"),
            format!("ALTER TABLE {table_name} ADD COLUMN IF NOT EXISTS close_authority Nullable(String)"),
        ]
    }
}

impl ClickHouseRow for TokenAccountClickHouseRow {
    fn table_name(&self) -> &'static str {
        Self::table()
    }

    fn partition_key(&self) -> String {
        self.partition_slot.to_string()
    }
}

fn format_datetime(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
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
