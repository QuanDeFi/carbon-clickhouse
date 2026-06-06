use std::{env, net::SocketAddr, sync::Arc};

use {
    carbon_core::{
        account::{AccountDecoder, AccountMetadata, AccountProcessorInputType},
        clickhouse::{ClickHouseAsyncInsertSettings, ClickHouseInsertSettings},
        error::{CarbonResult, Error as CarbonError},
        pipeline::ShutdownStrategy,
        processor::Processor,
    },
    carbon_log_metrics::LogMetrics,
    carbon_prometheus_metrics::{PrometheusMetrics, PrometheusServerConfig},
    carbon_rpc_block_crawler_datasource::{RpcBlockConfig, RpcBlockCrawler},
    carbon_token_program_decoder::{
        accounts::clickhouse as account_clickhouse,
        instructions::clickhouse as instruction_clickhouse, TokenProgramDecoder,
    },
    solana_account_decoder::UiAccountEncoding,
    solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcAccountInfoConfig},
    solana_commitment_config::CommitmentConfig,
    solana_pubkey::Pubkey,
    solana_transaction_status::UiTransactionEncoding,
};

const USDC_MINT: Pubkey = Pubkey::from_str_const("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const USDC_ACCOUNTS: [Pubkey; 4] = [
    USDC_MINT,
    Pubkey::from_str_const("BJE5MMbqXjVwjAF7oxwPYXnTXDyspzZyt4vwenNw5ruG"),
    Pubkey::from_str_const("7dGbd2QZcCKcTndnHcTL8q7SMVXAkp688NTQYwrRCrar"),
    Pubkey::from_str_const("11kzWyAwp9fG47nbgo47E2nKDxbi7hBwNcvDRbPFYk"),
];

#[tokio::main]
pub async fn main() -> CarbonResult<()> {
    dotenv::dotenv().ok();
    init_logger();

    let database_url = required_env("DATABASE_URL")?;
    let rpc_url = required_env("RPC_URL")?;

    let mut account_config =
        account_clickhouse::clickhouse_config_from_database_url(&database_url)?;
    account_config.source_name = "rpc_get_multiple_accounts".to_string();
    account_config.mode = "snapshot".to_string();
    account_config = clickhouse_config(account_config);
    account_clickhouse::TokenProgramClickHouseAccountsMigration::run(&account_config).await?;
    let mut account_processor = account_clickhouse::clickhouse_processor(account_config);
    let snapshot_slot = process_usdc_accounts(&rpc_url, &mut account_processor).await?;
    account_processor.finalize().await?;

    let mut instruction_config =
        instruction_clickhouse::clickhouse_config_from_database_url(&database_url)?;
    instruction_config.source_name = "rpc_block_crawler".to_string();
    instruction_config.mode = "live".to_string();
    instruction_config = clickhouse_config(instruction_config);
    instruction_clickhouse::TokenProgramClickHouseInstructionsMigration::run(&instruction_config)
        .await?;

    let start_slot = snapshot_slot.saturating_add(1);
    log::info!("Starting Token Program live block crawl from finalized slot {start_slot}");

    carbon_core::pipeline::Pipeline::builder()
        .datasource(block_crawler(rpc_url, start_slot))
        .metrics(Arc::new(LogMetrics::new_with_flush_interval(3)))
        .metrics(Arc::new(prometheus_metrics()?))
        .instruction(
            TokenProgramDecoder,
            instruction_clickhouse::clickhouse_processor(instruction_config),
        )
        .shutdown_strategy(ShutdownStrategy::Immediate)
        .build()?
        .run()
        .await?;

    Ok(())
}

async fn process_usdc_accounts(
    rpc_url: &str,
    processor: &mut account_clickhouse::TokenProgramClickHouseAccountProcessor,
) -> CarbonResult<u64> {
    let commitment = CommitmentConfig::finalized();
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

    log::info!(
        "USDC token program account snapshot complete at finalized slot {}",
        response.context.slot
    );
    Ok(response.context.slot)
}

async fn process_account(
    processor: &mut account_clickhouse::TokenProgramClickHouseAccountProcessor,
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

fn block_crawler(rpc_url: String, start_slot: u64) -> RpcBlockCrawler {
    RpcBlockCrawler::new(
        rpc_url,
        start_slot,
        None,
        None,
        RpcBlockConfig {
            commitment: Some(CommitmentConfig::finalized()),
            encoding: Some(UiTransactionEncoding::Binary),
            max_supported_transaction_version: Some(0),
            ..Default::default()
        },
        Some(1),
        None,
    )
}

fn clickhouse_config(
    mut config: carbon_core::clickhouse::ClickHouseConfig,
) -> carbon_core::clickhouse::ClickHouseConfig {
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
    config
}

fn init_logger() {
    let mut logger = env_logger::Builder::new();
    logger.filter_level(log::LevelFilter::Debug);
    if let Ok(log_level) = env::var("LOG_LEVEL") {
        logger.parse_filters(&log_level);
    }
    if let Ok(write_style) = env::var("RUST_LOG_STYLE") {
        logger.parse_write_style(&write_style);
    }
    logger.init();
}

fn prometheus_metrics() -> CarbonResult<PrometheusMetrics> {
    let addr = env::var("PROMETHEUS_METRICS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9465".to_string())
        .parse::<SocketAddr>()
        .map_err(|err| CarbonError::Custom(format!("Invalid PROMETHEUS_METRICS_ADDR: {err}")))?;
    Ok(PrometheusMetrics::with_server(
        PrometheusServerConfig::new().listen_addr(addr),
    ))
}

fn required_env(name: &str) -> CarbonResult<String> {
    env::var(name).map_err(|err| CarbonError::Custom(format!("{name} must be set ({err})")))
}

fn enabled(name: &str) -> bool {
    env::var(name)
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
}
