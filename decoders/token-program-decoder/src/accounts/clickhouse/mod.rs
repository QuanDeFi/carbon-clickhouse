pub mod mint_row;
pub mod multisig_row;
pub mod token_row;

pub use self::{mint_row::*, multisig_row::*, token_row::*};

use std::time::Duration;

use carbon_core::{
    account::{AccountMetadata, DecodedAccount},
    clickhouse::{
        rows::{ClickHouseRow, ClickHouseRowContext, ClickHouseRows, ClickHouseTable},
        ClickHouseAccountProcessor, ClickHouseAdmin, ClickHouseConfig, ClickHouseSchema,
    },
    error::CarbonResult,
};

use super::TokenProgramAccount;

pub const DEFAULT_DATABASE: &str = "default";
pub const DEFAULT_SOURCE_NAME: &str = "block_crawler";
pub const DEFAULT_MODE: &str = "backfill";
pub const DEFAULT_DECODER_VERSION: &str = "v1";
pub const DEFAULT_MAX_ROWS: usize = 500;
pub const DEFAULT_FLUSH_INTERVAL_MS: u64 = 1_000;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum TokenProgramClickHouseAccountRow {
    Mint(mint_row::MintAccountClickHouseRow),
    Multisig(multisig_row::MultisigAccountClickHouseRow),
    Token(token_row::TokenAccountClickHouseRow),
}

impl ClickHouseRow for TokenProgramClickHouseAccountRow {
    fn table_name(&self) -> &'static str {
        match self {
            Self::Mint(row) => row.table_name(),
            Self::Multisig(row) => row.table_name(),
            Self::Token(row) => row.table_name(),
        }
    }

    fn partition_key(&self) -> String {
        match self {
            Self::Mint(row) => row.partition_key(),
            Self::Multisig(row) => row.partition_key(),
            Self::Token(row) => row.partition_key(),
        }
    }
}

pub type TokenProgramClickHouseAccountProcessor = ClickHouseAccountProcessor<
    TokenProgramAccount,
    TokenProgramAccountWithClickHouseMetadata,
    TokenProgramClickHouseAccountRow,
>;

pub struct TokenProgramClickHouseAccountsMigration;

impl ClickHouseSchema for TokenProgramClickHouseAccountsMigration {
    fn operations(_config: &ClickHouseConfig) -> Vec<String> {
        vec![
            MintAccountClickHouseRow::create_table_sql(
                MintAccountClickHouseRow::DEFAULT_TABLE_NAME,
            ),
            MultisigAccountClickHouseRow::create_table_sql(
                MultisigAccountClickHouseRow::DEFAULT_TABLE_NAME,
            ),
            TokenAccountClickHouseRow::create_table_sql(
                TokenAccountClickHouseRow::DEFAULT_TABLE_NAME,
            ),
        ]
    }
}

impl TokenProgramClickHouseAccountsMigration {
    pub async fn run(config: &ClickHouseConfig) -> CarbonResult<()> {
        ClickHouseAdmin::new(config.clone())
            .execute_schema::<Self>()
            .await
    }
}

pub struct TokenProgramAccountWithClickHouseMetadata(
    pub DecodedAccount<TokenProgramAccount>,
    pub AccountMetadata,
);

impl From<(DecodedAccount<TokenProgramAccount>, AccountMetadata)>
    for TokenProgramAccountWithClickHouseMetadata
{
    fn from(value: (DecodedAccount<TokenProgramAccount>, AccountMetadata)) -> Self {
        Self(value.0, value.1)
    }
}

impl ClickHouseRows<TokenProgramClickHouseAccountRow>
    for TokenProgramAccountWithClickHouseMetadata
{
    fn clickhouse_rows(
        &self,
        context: &ClickHouseRowContext,
    ) -> Vec<TokenProgramClickHouseAccountRow> {
        let TokenProgramAccountWithClickHouseMetadata(decoded_account, metadata) = self;

        match &decoded_account.data {
            TokenProgramAccount::Mint(account) => {
                vec![TokenProgramClickHouseAccountRow::Mint(
                    MintAccountClickHouseRow::from_parts(
                        *account.clone(),
                        decoded_account,
                        metadata,
                        context,
                    ),
                )]
            }
            TokenProgramAccount::Multisig(account) => {
                vec![TokenProgramClickHouseAccountRow::Multisig(
                    MultisigAccountClickHouseRow::from_parts(
                        *account.clone(),
                        decoded_account,
                        metadata,
                        context,
                    ),
                )]
            }
            TokenProgramAccount::Token(account) => {
                vec![TokenProgramClickHouseAccountRow::Token(
                    TokenAccountClickHouseRow::from_parts(
                        *account.clone(),
                        decoded_account,
                        metadata,
                        context,
                    ),
                )]
            }
        }
    }
}

pub fn clickhouse_config_from_database_url(database_url: &str) -> CarbonResult<ClickHouseConfig> {
    ClickHouseConfig::from_database_url(
        database_url,
        DEFAULT_DATABASE.to_string(),
        MintAccountClickHouseRow::DEFAULT_TABLE_NAME.to_string(),
        DEFAULT_SOURCE_NAME.to_string(),
        DEFAULT_MODE.to_string(),
        DEFAULT_DECODER_VERSION.to_string(),
        DEFAULT_MAX_ROWS,
        Duration::from_millis(DEFAULT_FLUSH_INTERVAL_MS),
    )
}

pub async fn bootstrap_clickhouse_from_database_url(
    database_url: &str,
) -> CarbonResult<ClickHouseConfig> {
    let config = clickhouse_config_from_database_url(database_url)?;
    TokenProgramClickHouseAccountsMigration::run(&config).await?;
    Ok(config)
}

pub fn clickhouse_processor(config: ClickHouseConfig) -> TokenProgramClickHouseAccountProcessor {
    ClickHouseAccountProcessor::new(config)
}

pub async fn setup_clickhouse(
    database_url: &str,
) -> CarbonResult<TokenProgramClickHouseAccountProcessor> {
    let config = bootstrap_clickhouse_from_database_url(database_url).await?;
    Ok(clickhouse_processor(config))
}
