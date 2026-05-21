use std::env;

use {
    carbon_core::{
        account::{AccountDecoder, AccountMetadata, AccountProcessorInputType},
        clickhouse::{ClickHouseAsyncInsertSettings, ClickHouseInsertSettings},
        error::{CarbonResult, Error as CarbonError},
        processor::Processor,
    },
    carbon_token_program_decoder::{
        accounts::clickhouse::{
            clickhouse_config_from_database_url, clickhouse_processor,
            TokenProgramClickHouseAccountProcessor, TokenProgramClickHouseAccountsMigration,
        },
        TokenProgramDecoder,
    },
    solana_account_decoder::UiAccountEncoding,
    solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcAccountInfoConfig},
    solana_commitment_config::CommitmentConfig,
    solana_pubkey::Pubkey,
};

const USDC_ACCOUNTS: [Pubkey; 4] = [
    Pubkey::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    Pubkey::from_str_const("BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG"),
    Pubkey::from_str_const("7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar"),
    Pubkey::from_str_const("11kzWyAwp9fG47nbgo47E2nKDxbi7hBwNcvDRbPFYk"),
];

#[tokio::main]
pub async fn main() -> CarbonResult<()> {
    dotenv::dotenv().ok();
    init_logger();

    let database_url = required_env("DATABASE_URL")?;
    let mut config = clickhouse_config_from_database_url(&database_url)?;
    if enabled("CLICKHOUSE_ASYNC_INSERT") {
        config = config.with_insert_settings(ClickHouseInsertSettings::AsyncWait(
            ClickHouseAsyncInsertSettings {
                busy_timeout_ms: Some(1_000),
                max_data_size: None,
                max_query_number: None,
                deduplicate: None,
            },
        ));
    }

    TokenProgramClickHouseAccountsMigration::run(&config).await?;
    let mut processor = clickhouse_processor(config);
    process_usdc_accounts(&required_env("RPC_URL")?, &mut processor).await?;
    processor.finalize().await?;

    log::info!("USDC token program snapshot complete");
    Ok(())
}

async fn process_usdc_accounts(
    rpc_url: &str,
    processor: &mut TokenProgramClickHouseAccountProcessor,
) -> CarbonResult<()> {
    let commitment = CommitmentConfig::confirmed();
    let response = RpcClient::new_with_commitment(rpc_url.to_string(), commitment)
        .get_multiple_ui_accounts_with_config(
            &USDC_ACCOUNTS,
            RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                commitment: Some(commitment),
                ..Default::default()
            },
        )
        .await
        .map_err(|err| {
            CarbonError::FailedToConsumeDatasource(format!("Failed to fetch USDC accounts: {err}"))
        })?;

    for (pubkey, account) in USDC_ACCOUNTS.into_iter().zip(response.value) {
        let Some(account) = account.and_then(|account| account.decode()) else {
            return Err(CarbonError::FailedToConsumeDatasource(format!(
                "USDC account {pubkey} was missing or could not be decoded"
            )));
        };
        process_account(processor, pubkey, response.context.slot, &account).await?;
    }

    Ok(())
}

async fn process_account(
    processor: &mut TokenProgramClickHouseAccountProcessor,
    pubkey: Pubkey,
    slot: u64,
    account: &solana_account::Account,
) -> CarbonResult<()> {
    let metadata = AccountMetadata {
        slot,
        pubkey,
        transaction_signature: None,
    };
    if let Some(decoded_account) = TokenProgramDecoder.decode_account(account) {
        processor
            .process(&AccountProcessorInputType {
                metadata: &metadata,
                decoded_account: &decoded_account,
                raw_account: account,
            })
            .await?;
    }
    Ok(())
}

fn init_logger() {
    let mut logger = env_logger::Builder::new();
    logger.filter_level(log::LevelFilter::Info);
    if let Ok(log_level) = env::var("LOG_LEVEL") {
        logger.parse_filters(&log_level);
    }
    logger.init();
}

fn required_env(name: &str) -> CarbonResult<String> {
    env::var(name).map_err(|err| CarbonError::Custom(format!("{name} must be set ({err})")))
}

fn enabled(name: &str) -> bool {
    env::var(name)
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
}
