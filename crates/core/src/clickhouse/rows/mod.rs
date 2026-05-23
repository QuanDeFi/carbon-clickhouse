use crate::{
    account::{AccountMetadata, DecodedAccount},
    clickhouse::admin::ClickHouseColumnSpec,
    error::CarbonResult,
    instruction::InstructionMetadata,
};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

pub trait ClickHouseTable {
    fn table() -> &'static str;
    fn columns() -> Vec<&'static str>;
    fn create_table_sql(table_name: &str) -> String;
}

pub trait ClickHouseRow:
    serde::Serialize + Clone + Send + Sync + std::fmt::Debug + 'static
{
    fn table_name(&self) -> &'static str;
    fn partition_key(&self) -> String;

    fn to_json_line(&self) -> CarbonResult<String> {
        serde_json::to_string(self).map_err(|e| crate::error::Error::Custom(e.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseRowContext {
    pub source_name: String,
    pub mode: String,
    pub decoder_version: String,
}

pub trait ClickHouseRows<R: ClickHouseRow>: Send + Sync + 'static {
    fn clickhouse_rows(&self, context: &ClickHouseRowContext) -> Vec<R>;
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ClickHouseInstructionLandingMetadata {
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

impl ClickHouseInstructionLandingMetadata {
    pub fn new(
        program_id_bytes: &[u8],
        program_id: String,
        family_name: &'static str,
        instruction_type: &'static str,
        metadata: &InstructionMetadata,
        context: &ClickHouseRowContext,
    ) -> Self {
        let ingest_ts = Utc::now();
        let chain_time = metadata
            .transaction_metadata
            .block_time
            .and_then(DateTime::<Utc>::from_timestamp_secs);
        let partition_time = chain_time.unwrap_or(ingest_ts);
        let signature = metadata.transaction_metadata.signature.to_string();

        Self {
            program_id,
            family_name: family_name.to_string(),
            instruction_type: instruction_type.to_string(),
            instruction_id: deterministic_instruction_id(
                program_id_bytes,
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
        self.partition_time.get(..4).unwrap_or("0000").to_string()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ClickHouseEventLandingMetadata {
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
}

impl ClickHouseEventLandingMetadata {
    pub fn new(
        program_id_bytes: &[u8],
        program_id: String,
        family_name: &'static str,
        event_type: &'static str,
        event_seq: u32,
        metadata: &InstructionMetadata,
        context: &ClickHouseRowContext,
    ) -> Self {
        let ingest_ts = Utc::now();
        let chain_time = metadata
            .transaction_metadata
            .block_time
            .and_then(DateTime::<Utc>::from_timestamp_secs);
        let partition_time = chain_time.unwrap_or(ingest_ts);
        let signature = metadata.transaction_metadata.signature.to_string();

        Self {
            program_id,
            family_name: family_name.to_string(),
            event_type: event_type.to_string(),
            event_id: deterministic_event_id(
                program_id_bytes,
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
        }
    }

    pub fn partition_key(&self) -> String {
        self.partition_time.get(..4).unwrap_or("0000").to_string()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ClickHouseAccountLandingMetadata {
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
}

impl ClickHouseAccountLandingMetadata {
    pub fn new<T>(
        program_id_bytes: &[u8],
        program_id: String,
        family_name: &'static str,
        account_type: &'static str,
        decoded_account: &DecodedAccount<T>,
        metadata: &AccountMetadata,
        context: &ClickHouseRowContext,
    ) -> Self {
        let ingest_ts = Utc::now();
        let partition_slot = metadata.slot / 1_000_000;

        Self {
            program_id,
            family_name: family_name.to_string(),
            account_type: account_type.to_string(),
            account_id: deterministic_account_id(
                program_id_bytes,
                metadata.pubkey.as_ref(),
                metadata.slot,
                account_type,
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
        }
    }

    pub fn partition_key(&self) -> String {
        self.partition_slot.to_string()
    }
}

pub const CLICKHOUSE_INSTRUCTION_COMMON_COLUMNS: &[ClickHouseColumnSpec] = &[
    ClickHouseColumnSpec::new("program_id", "String"),
    ClickHouseColumnSpec::new("family_name", "String"),
    ClickHouseColumnSpec::new("instruction_type", "String"),
    ClickHouseColumnSpec::new("instruction_id", "String"),
    ClickHouseColumnSpec::new("slot", "UInt64"),
    ClickHouseColumnSpec::new("signature", "String"),
    ClickHouseColumnSpec::new("instruction_index", "UInt32"),
    ClickHouseColumnSpec::new("stack_height", "UInt32"),
    ClickHouseColumnSpec::new("absolute_path", "Array(UInt8)"),
    ClickHouseColumnSpec::new("source_name", "String"),
    ClickHouseColumnSpec::new("mode", "String"),
    ClickHouseColumnSpec::new("decoder_version", "String"),
    ClickHouseColumnSpec::new("ingest_ts", "DateTime64(3, 'UTC')"),
    ClickHouseColumnSpec::new("chain_time", "Nullable(DateTime64(3, 'UTC'))"),
    ClickHouseColumnSpec::new("partition_time", "DateTime64(3, 'UTC')"),
    ClickHouseColumnSpec::new("block_hash", "Nullable(String)"),
    ClickHouseColumnSpec::new("tx_index", "Nullable(UInt64)"),
];

pub const CLICKHOUSE_EVENT_COMMON_COLUMNS: &[ClickHouseColumnSpec] = &[
    ClickHouseColumnSpec::new("program_id", "String"),
    ClickHouseColumnSpec::new("family_name", "String"),
    ClickHouseColumnSpec::new("event_type", "String"),
    ClickHouseColumnSpec::new("event_id", "String"),
    ClickHouseColumnSpec::new("slot", "UInt64"),
    ClickHouseColumnSpec::new("signature", "String"),
    ClickHouseColumnSpec::new("instruction_index", "UInt32"),
    ClickHouseColumnSpec::new("stack_height", "UInt32"),
    ClickHouseColumnSpec::new("absolute_path", "Array(UInt8)"),
    ClickHouseColumnSpec::new("event_seq", "UInt32"),
    ClickHouseColumnSpec::new("source_name", "String"),
    ClickHouseColumnSpec::new("mode", "String"),
    ClickHouseColumnSpec::new("decoder_version", "String"),
    ClickHouseColumnSpec::new("ingest_ts", "DateTime64(3, 'UTC')"),
    ClickHouseColumnSpec::new("chain_time", "Nullable(DateTime64(3, 'UTC'))"),
    ClickHouseColumnSpec::new("partition_time", "DateTime64(3, 'UTC')"),
    ClickHouseColumnSpec::new("block_hash", "Nullable(String)"),
    ClickHouseColumnSpec::new("tx_index", "Nullable(UInt64)"),
];

pub const CLICKHOUSE_ACCOUNT_COMMON_COLUMNS: &[ClickHouseColumnSpec] = &[
    ClickHouseColumnSpec::new("program_id", "String"),
    ClickHouseColumnSpec::new("family_name", "String"),
    ClickHouseColumnSpec::new("account_type", "String"),
    ClickHouseColumnSpec::new("account_id", "String"),
    ClickHouseColumnSpec::new("slot", "UInt64"),
    ClickHouseColumnSpec::new("pubkey", "String"),
    ClickHouseColumnSpec::new("transaction_signature", "Nullable(String)"),
    ClickHouseColumnSpec::new("lamports", "UInt64"),
    ClickHouseColumnSpec::new("owner", "String"),
    ClickHouseColumnSpec::new("executable", "Bool"),
    ClickHouseColumnSpec::new("rent_epoch", "UInt64"),
    ClickHouseColumnSpec::new("source_name", "String"),
    ClickHouseColumnSpec::new("mode", "String"),
    ClickHouseColumnSpec::new("decoder_version", "String"),
    ClickHouseColumnSpec::new("ingest_ts", "DateTime64(3, 'UTC')"),
    ClickHouseColumnSpec::new("partition_slot", "UInt64"),
];

pub fn deterministic_event_id(
    program_id: &[u8],
    signature: &str,
    absolute_path: &[u8],
    event_type: &str,
    event_seq: u32,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(program_id);
    hasher.update(signature.as_bytes());
    hasher.update(absolute_path);
    hasher.update(event_type.as_bytes());
    hasher.update(event_seq.to_le_bytes());
    hex_digest(hasher.finalize())
}

pub fn deterministic_instruction_id(
    program_id: &[u8],
    signature: &str,
    absolute_path: &[u8],
    instruction_type: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(program_id);
    hasher.update(signature.as_bytes());
    hasher.update(absolute_path);
    hasher.update(instruction_type.as_bytes());
    hex_digest(hasher.finalize())
}

pub fn deterministic_account_id(
    program_id: &[u8],
    pubkey: &[u8],
    slot: u64,
    account_type: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(program_id);
    hasher.update(pubkey);
    hasher.update(slot.to_le_bytes());
    hasher.update(account_type.as_bytes());
    hex_digest(hasher.finalize())
}

fn hex_digest(digest: impl IntoIterator<Item = u8>) -> String {
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

pub fn format_datetime(value: DateTime<Utc>) -> String {
    value.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
}

pub fn clickhouse_enum_variant<T>(value: &T) -> String
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

pub fn clickhouse_debug_variant<T>(value: &T) -> String
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

#[macro_export]
macro_rules! __impl_clickhouse_landing_row {
    ($row:ident, $table_options:expr) => {
        impl $row {
            fn column_specs() -> Vec<$crate::clickhouse::ClickHouseColumnSpec> {
                let mut columns = Self::COMMON_COLUMNS.to_vec();
                columns.extend_from_slice(Self::PAYLOAD_COLUMNS);
                columns
            }

            fn table_options() -> $crate::clickhouse::ClickHouseTableOptions {
                ($table_options)()
            }

            pub fn migration_operations(table_name: &str) -> Vec<String> {
                let columns = Self::column_specs();
                $crate::clickhouse::clickhouse_migration_operations(
                    table_name,
                    &columns,
                    &Self::table_options(),
                )
            }

            pub fn managed_tables(
                table_name: &str,
            ) -> Vec<$crate::clickhouse::ClickHouseManagedTable> {
                let columns = Self::column_specs();
                $crate::clickhouse::clickhouse_managed_tables(
                    table_name,
                    &columns,
                    &Self::table_options(),
                )
            }
        }

        impl $crate::clickhouse::rows::ClickHouseTable for $row {
            fn table() -> &'static str {
                Self::DEFAULT_TABLE_NAME
            }

            fn columns() -> Vec<&'static str> {
                let columns = Self::column_specs();
                $crate::clickhouse::clickhouse_column_names(&columns)
            }

            fn create_table_sql(table_name: &str) -> String {
                let columns = Self::column_specs();
                let options = Self::table_options();
                $crate::clickhouse::clickhouse_create_table_sql(
                    table_name,
                    &columns,
                    &options.engine,
                    options.local_engine.is_none(),
                    &options,
                )
            }
        }

        impl $crate::clickhouse::rows::ClickHouseRow for $row {
            fn table_name(&self) -> &'static str {
                <$row as $crate::clickhouse::rows::ClickHouseTable>::table()
            }

            fn partition_key(&self) -> String {
                self.metadata.partition_key()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_clickhouse_instruction_row {
    ($row:ident, $table_options:expr) => {
        $crate::__impl_clickhouse_landing_row!($row, $table_options);
    };
}

#[macro_export]
macro_rules! impl_clickhouse_event_row {
    ($row:ident, $table_options:expr) => {
        $crate::__impl_clickhouse_landing_row!($row, $table_options);
    };
}

#[macro_export]
macro_rules! impl_clickhouse_account_row {
    ($row:ident, $table_options:expr) => {
        $crate::__impl_clickhouse_landing_row!($row, $table_options);
    };
}

#[macro_export]
macro_rules! clickhouse_row_dispatch {
    (
        instruction $enum:ident,
        $migration:ident,
        $metadata:ident,
        $instruction:ident,
        {
            $($variant:ident => $row:ty),+ $(,)?
        }
        events $event_enum:ident {
            $($event_variant:ident : $source_event_variant:ident => $event_row:ty),+ $(,)?
        }
    ) => {
        $crate::clickhouse_row_dispatch!(
            $enum,
            $migration,
            {
                $($variant => $row,)+
                $($event_variant => $event_row,)+
            }
        );

        impl $crate::clickhouse::rows::ClickHouseRows<$enum> for $metadata {
            fn clickhouse_rows(
                &self,
                context: &$crate::clickhouse::rows::ClickHouseRowContext,
            ) -> Vec<$enum> {
                let $metadata(instruction, metadata, _accounts) = self;

                match instruction {
                    $(
                        $instruction::$variant { data, .. } => {
                            vec![$enum::$variant(<$row>::from_parts(
                                data.clone(),
                                metadata,
                                context,
                            ))]
                        }
                    )+
                    $instruction::CpiEvent { data, .. } => match data {
                        $(
                            $event_enum::$source_event_variant(event) => {
                                vec![$enum::$event_variant(<$event_row>::from_parts(
                                    event.clone(),
                                    metadata,
                                    context,
                                ))]
                            }
                        )+
                    },
                }
            }
        }
    };
    (
        instruction $enum:ident,
        $migration:ident,
        $metadata:ident,
        $instruction:ident,
        {
            $($variant:ident => $row:ty),+ $(,)?
        }
    ) => {
        $crate::clickhouse_row_dispatch!($enum, $migration, { $($variant => $row),+ });

        impl $crate::clickhouse::rows::ClickHouseRows<$enum> for $metadata {
            fn clickhouse_rows(
                &self,
                context: &$crate::clickhouse::rows::ClickHouseRowContext,
            ) -> Vec<$enum> {
                let $metadata(instruction, metadata, _accounts) = self;

                match instruction {
                    $(
                        $instruction::$variant { data, .. } => {
                            vec![$enum::$variant(<$row>::from_parts(
                                data.clone(),
                                metadata,
                                context,
                            ))]
                        }
                    )+
                }
            }
        }
    };
    (
        account $enum:ident,
        $migration:ident,
        $metadata:ident,
        $account:ident,
        {
            $($variant:ident => $row:ty),+ $(,)?
        }
    ) => {
        $crate::clickhouse_row_dispatch!($enum, $migration, { $($variant => $row),+ });

        impl $crate::clickhouse::rows::ClickHouseRows<$enum> for $metadata {
            fn clickhouse_rows(
                &self,
                context: &$crate::clickhouse::rows::ClickHouseRowContext,
            ) -> Vec<$enum> {
                let $metadata(decoded_account, metadata) = self;

                match &decoded_account.data {
                    $(
                        $account::$variant(account) => {
                            vec![$enum::$variant(<$row>::from_parts(
                                *account.clone(),
                                decoded_account,
                                metadata,
                                context,
                            ))]
                        }
                    )+
                }
            }
        }
    };
    (
        $enum:ident,
        $migration:ident,
        {
            $($variant:ident => $row:ty),+ $(,)?
        }
    ) => {
        impl $crate::clickhouse::rows::ClickHouseRow for $enum {
            fn table_name(&self) -> &'static str {
                match self {
                    $(
                        Self::$variant(row) => row.table_name(),
                    )+
                }
            }

            fn partition_key(&self) -> String {
                match self {
                    $(
                        Self::$variant(row) => row.partition_key(),
                    )+
                }
            }
        }

        impl $crate::clickhouse::ClickHouseSchema for $migration {
            fn operations(_config: &$crate::clickhouse::ClickHouseConfig) -> Vec<String> {
                let mut operations = Vec::new();
                $(
                    operations.extend(<$row>::migration_operations(
                        <$row as $crate::clickhouse::rows::ClickHouseTable>::table(),
                    ));
                )+
                operations
            }

            fn managed_tables(
                _config: &$crate::clickhouse::ClickHouseConfig,
            ) -> Vec<$crate::clickhouse::ClickHouseManagedTable> {
                let mut tables = Vec::new();
                $(
                    tables.extend(<$row>::managed_tables(
                        <$row as $crate::clickhouse::rows::ClickHouseTable>::table(),
                    ));
                )+
                tables
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::{deterministic_account_id, deterministic_event_id, deterministic_instruction_id};

    #[test]
    fn deterministic_event_id_is_stable() {
        let first = deterministic_event_id(b"program", "sig", &[1, 2, 3], "swap_event", 0);
        let second = deterministic_event_id(b"program", "sig", &[1, 2, 3], "swap_event", 0);
        assert_eq!(first, second);
    }

    #[test]
    fn deterministic_instruction_id_is_stable() {
        let first = deterministic_instruction_id(b"program", "sig", &[1, 2, 3], "swap");
        let second = deterministic_instruction_id(b"program", "sig", &[1, 2, 3], "swap");
        assert_eq!(first, second);
    }

    #[test]
    fn deterministic_account_id_is_stable() {
        let first = deterministic_account_id(b"program", b"pubkey", 42, "mint");
        let second = deterministic_account_id(b"program", b"pubkey", 42, "mint");
        assert_eq!(first, second);
    }
}
