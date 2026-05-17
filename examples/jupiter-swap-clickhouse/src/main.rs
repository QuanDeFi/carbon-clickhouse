use std::{collections::HashSet, env, net::SocketAddr, sync::{Arc, Mutex}};

use carbon_core::{account::{AccountDecoder, AccountMetadata, AccountProcessorInputType}, clickhouse::{ClickHouseAsyncInsertSettings, ClickHouseConfig, ClickHouseInsertSettings}, error::{CarbonResult, Error as CarbonError}, instruction::InstructionProcessorInputType, pipeline::ShutdownStrategy, processor::Processor};
use carbon_jupiter_swap_decoder::{accounts::clickhouse::{bootstrap_clickhouse_from_database_url as bootstrap_accounts_clickhouse, clickhouse_processor as account_processor}, instructions::{clickhouse::{bootstrap_clickhouse_from_database_url, clickhouse_processor}, JupiterSwapInstruction}, JupiterSwapDecoder};
use carbon_log_metrics::LogMetrics;
use carbon_prometheus_metrics::{PrometheusMetrics, PrometheusServerConfig};
use carbon_rpc_block_crawler_datasource::{RpcBlockConfig, RpcBlockCrawler};
use clap::Parser;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::Pubkey;
use solana_transaction_status::UiTransactionEncoding;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)] start_slot: Option<u64>,
    #[arg(short, long)] end_slot: Option<u64>,
    #[arg(long)] include_token_ledger_accounts: bool,
}

#[derive(Clone, Default)]
struct TokenLedgerPubkeys(Arc<Mutex<HashSet<Pubkey>>>);

impl TokenLedgerPubkeys {
    fn sorted(&self) -> CarbonResult<Vec<Pubkey>> {
        let mut pubkeys = self.0.lock()
            .map_err(|_| err("TokenLedger pubkey mutex was poisoned"))?
            .iter().copied().collect::<Vec<_>>();
        pubkeys.sort_by_key(|pubkey| pubkey.to_string());
        Ok(pubkeys)
    }
}

impl Processor<InstructionProcessorInputType<'_, JupiterSwapInstruction>> for TokenLedgerPubkeys {
    async fn process(
        &mut self,
        input: &InstructionProcessorInputType<'_, JupiterSwapInstruction>,
    ) -> CarbonResult<()> {
        if let Some(pubkey) = token_ledger_pubkey(input.decoded_instruction) {
            self.0.lock()
                .map_err(|_| err("TokenLedger pubkey mutex was poisoned"))?
                .insert(pubkey);
        }
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> CarbonResult<()> {
    dotenv::dotenv().ok();
    init_logger();

    let args = Args::parse();
    let database_url = required_env("DATABASE_URL")?;
    let rpc_url = required_env("RPC_URL")?;
    let token_ledger_pubkeys = TokenLedgerPubkeys::default();
    let instruction_config = clickhouse_config(bootstrap_clickhouse_from_database_url(&database_url).await?);

    carbon_core::pipeline::Pipeline::builder()
        .datasource(block_crawler(&args, rpc_url.clone())?)
        .metrics(Arc::new(LogMetrics::new_with_flush_interval(3)))
        .metrics(Arc::new(prometheus_metrics()?))
        .instruction(JupiterSwapDecoder, clickhouse_processor(instruction_config))
        .instruction(JupiterSwapDecoder, token_ledger_pubkeys.clone())
        .shutdown_strategy(ShutdownStrategy::Immediate)
        .build()?
        .run()
        .await?;

    let pubkeys = token_ledger_pubkeys.sorted()?;
    log::info!("TokenLedger pubkeys collected: {}", pubkeys.len());
    if args.include_token_ledger_accounts {
        write_token_ledger_accounts(&database_url, &rpc_url, &pubkeys).await?;
    }
    Ok(())
}

async fn write_token_ledger_accounts(
    database_url: &str,
    rpc_url: &str,
    pubkeys: &[Pubkey],
) -> CarbonResult<()> {
    if pubkeys.is_empty() {
        log::warn!("TokenLedger account smoke requested, but no pubkeys were collected.");
        return Ok(());
    }

    let mut config = clickhouse_config(bootstrap_accounts_clickhouse(database_url).await?);
    config.source_name = "exact_token_ledger_accounts".to_string();

    let rpc = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let mut processor = account_processor(config);
    let mut processed = 0usize;

    for chunk in pubkeys.chunks(100) {
        let response = rpc
            .get_multiple_accounts_with_commitment(chunk, CommitmentConfig::confirmed())
            .await
            .map_err(|error| {
                CarbonError::FailedToConsumeDatasource(format!(
                    "Failed to fetch TokenLedger accounts: {error}"
                ))
            })?;

        for (pubkey, account) in chunk.iter().copied().zip(response.value).filter_map(|(p, a)| Some((p, a?))) {
            let Some(decoded_account) = JupiterSwapDecoder.decode_account(&account) else { continue };
            let metadata = AccountMetadata {
                slot: response.context.slot,
                pubkey,
                transaction_signature: None,
            };
            processor
                .process(&AccountProcessorInputType {
                    metadata: &metadata,
                    decoded_account: &decoded_account,
                    raw_account: &account,
                })
                .await?;
            processed += 1;
        }
    }

    processor.finalize().await?;
    log::info!(
        "TokenLedger account smoke complete. requested: {}, processed: {}",
        pubkeys.len(),
        processed
    );
    Ok(())
}

fn token_ledger_pubkey(instruction: &JupiterSwapInstruction) -> Option<Pubkey> {
    match instruction {
        JupiterSwapInstruction::CreateTokenLedger { accounts, .. } => Some(accounts.token_ledger),
        JupiterSwapInstruction::SetTokenLedger { accounts, .. } => Some(accounts.token_ledger),
        JupiterSwapInstruction::RouteWithTokenLedger { accounts, .. } => Some(accounts.token_ledger),
        JupiterSwapInstruction::SharedAccountsRouteWithTokenLedger { accounts, .. } => {
            Some(accounts.token_ledger)
        }
        _ => None,
    }
}

fn block_crawler(args: &Args, rpc_url: String) -> CarbonResult<RpcBlockCrawler> {
    Ok(RpcBlockCrawler::new(
        rpc_url,
        slot_arg(args.start_slot, "BLOCK_CRAWLER_START_SLOT")?,
        args.end_slot
            .or_else(|| env::var("BLOCK_CRAWLER_END_SLOT").ok()?.parse().ok()),
        None,
        RpcBlockConfig { encoding: Some(UiTransactionEncoding::Binary), max_supported_transaction_version: Some(0), ..Default::default() },
        Some(1),
        None,
    ))
}

fn clickhouse_config(config: ClickHouseConfig) -> ClickHouseConfig {
    if enabled("CLICKHOUSE_ASYNC_INSERT") {
        config.with_insert_settings(ClickHouseInsertSettings::AsyncWait(ClickHouseAsyncInsertSettings::default()))
    } else {
        config
    }
}

fn prometheus_metrics() -> CarbonResult<PrometheusMetrics> {
    let addr = env::var("PROMETHEUS_METRICS_ADDR").unwrap_or_else(|_| "0.0.0.0:9464".to_string())
        .parse::<SocketAddr>().map_err(err)?;
    Ok(PrometheusMetrics::with_server(PrometheusServerConfig::new().listen_addr(addr)))
}

fn slot_arg(value: Option<u64>, env_name: &str) -> CarbonResult<u64> {
    value
        .or_else(|| env::var(env_name).ok()?.parse().ok())
        .ok_or_else(|| err(format!("{env_name} is required")))
}

fn enabled(name: &str) -> bool {
    matches!(
        env::var(name).unwrap_or_default().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "y" | "on"
    )
}

fn init_logger() {
    let mut logger = env_logger::Builder::new();
    logger.filter_level(log::LevelFilter::Debug);
    if let Ok(log_level) = env::var("LOG_LEVEL") { logger.parse_filters(&log_level); }
    logger.init();
}

fn required_env(name: &str) -> CarbonResult<String> {
    env::var(name).map_err(err)
}

fn err(error: impl std::fmt::Display) -> CarbonError {
    CarbonError::Custom(error.to_string())
}
