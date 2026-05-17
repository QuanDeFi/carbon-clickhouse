use std::{collections::HashSet, env, net::SocketAddr, sync::Arc};

use carbon_core::{
    account::{AccountDecoder, AccountMetadata, AccountProcessorInputType},
    clickhouse::{ClickHouseAsyncInsertSettings, ClickHouseConfig, ClickHouseInsertSettings},
    error::{CarbonResult, Error as CarbonError},
    instruction::InstructionProcessorInputType,
    pipeline::ShutdownStrategy,
    processor::Processor,
};
use carbon_jupiter_swap_decoder::{
    accounts::clickhouse::{
        bootstrap_clickhouse_from_database_url as bootstrap_accounts_clickhouse,
        clickhouse_processor as account_processor, JupiterSwapClickHouseAccountProcessor,
    },
    instructions::{
        clickhouse::{bootstrap_clickhouse_from_database_url, clickhouse_processor},
        JupiterSwapInstruction,
    },
    JupiterSwapDecoder,
};
use carbon_log_metrics::LogMetrics;
use carbon_prometheus_metrics::{PrometheusMetrics, PrometheusServerConfig};
use carbon_rpc_block_crawler_datasource::{RpcBlockConfig, RpcBlockCrawler};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_commitment_config::CommitmentConfig;
use solana_pubkey::Pubkey;
use solana_transaction_status::UiTransactionEncoding;

const DEFAULT_HEAD_LAG_SLOTS: u64 = 3;

struct TokenLedgerPubkeys {
    seen: HashSet<Pubkey>,
    rpc: RpcClient,
    processor: JupiterSwapClickHouseAccountProcessor,
}

impl TokenLedgerPubkeys {
    fn new(rpc_url: &str, processor: JupiterSwapClickHouseAccountProcessor) -> Self {
        Self {
            seen: HashSet::new(),
            rpc: RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed()),
            processor,
        }
    }

    async fn fetch(&mut self, pubkey: Pubkey) -> CarbonResult<()> {
        let response = self
            .rpc
            .get_account_with_commitment(&pubkey, CommitmentConfig::confirmed())
            .await
            .map_err(|error| {
                CarbonError::FailedToConsumeDatasource(format!(
                    "Failed to fetch TokenLedger account {pubkey}: {error}"
                ))
            })?;

        let Some(account) = response.value else {
            return Ok(());
        };
        let Some(decoded_account) = JupiterSwapDecoder.decode_account(&account) else {
            return Ok(());
        };
        let metadata = AccountMetadata {
            slot: response.context.slot,
            pubkey,
            transaction_signature: None,
        };
        self.processor
            .process(&AccountProcessorInputType {
                metadata: &metadata,
                decoded_account: &decoded_account,
                raw_account: &account,
            })
            .await?;
        self.processor.flush().await?;
        log::info!("TokenLedger account fetched: {pubkey}");
        Ok(())
    }
}

impl Processor<InstructionProcessorInputType<'_, JupiterSwapInstruction>> for TokenLedgerPubkeys {
    async fn process(
        &mut self,
        input: &InstructionProcessorInputType<'_, JupiterSwapInstruction>,
    ) -> CarbonResult<()> {
        if let Some(pubkey) = token_ledger_pubkey(input.decoded_instruction) {
            if self.seen.insert(pubkey) {
                self.fetch(pubkey).await?;
            }
        }
        Ok(())
    }

    async fn finalize(&mut self) -> CarbonResult<()> {
        self.processor.finalize().await?;
        log::info!("TokenLedger pubkeys collected: {}", self.seen.len());
        Ok(())
    }
}

#[tokio::main]
pub async fn main() -> CarbonResult<()> {
    dotenv::dotenv().ok();
    init_logger();

    let database_url = required_env("DATABASE_URL")?;
    let rpc_url = required_env("RPC_URL")?;
    let start_slot = optional_env_u64("BLOCK_CRAWLER_START_SLOT")?;
    let end_slot = optional_env_u64("BLOCK_CRAWLER_END_SLOT")?;
    let head_live_mode = start_slot.is_none() && end_slot.is_none();

    let instruction_config =
        clickhouse_config(bootstrap_clickhouse_from_database_url(&database_url).await?);
    let token_ledger_processor = if head_live_mode {
        let mut config = clickhouse_config(bootstrap_accounts_clickhouse(&database_url).await?);
        config.source_name = "exact_token_ledger_accounts".to_string();
        Some(TokenLedgerPubkeys::new(&rpc_url, account_processor(config)))
    } else {
        None
    };

    let mut pipeline = carbon_core::pipeline::Pipeline::builder()
        .datasource(block_crawler(rpc_url.clone(), start_slot, end_slot).await?)
        .metrics(Arc::new(LogMetrics::new_with_flush_interval(3)))
        .metrics(Arc::new(prometheus_metrics()?))
        .instruction(JupiterSwapDecoder, clickhouse_processor(instruction_config))
        .shutdown_strategy(ShutdownStrategy::Immediate);
    if let Some(processor) = token_ledger_processor {
        pipeline = pipeline.instruction(JupiterSwapDecoder, processor);
    }
    pipeline.build()?.run().await?;
    Ok(())
}

fn token_ledger_pubkey(instruction: &JupiterSwapInstruction) -> Option<Pubkey> {
    match instruction {
        JupiterSwapInstruction::CreateTokenLedger { accounts, .. } => Some(accounts.token_ledger),
        JupiterSwapInstruction::SetTokenLedger { accounts, .. } => Some(accounts.token_ledger),
        JupiterSwapInstruction::RouteWithTokenLedger { accounts, .. } => {
            Some(accounts.token_ledger)
        }
        JupiterSwapInstruction::SharedAccountsRouteWithTokenLedger { accounts, .. } => {
            Some(accounts.token_ledger)
        }
        _ => None,
    }
}

async fn block_crawler(
    rpc_url: String,
    start_slot: Option<u64>,
    end_slot: Option<u64>,
) -> CarbonResult<RpcBlockCrawler> {
    let start_slot = if let Some(start_slot) = start_slot {
        start_slot
    } else if end_slot.is_none() {
        let head_lag_slots =
            optional_env_u64("BLOCK_CRAWLER_HEAD_LAG_SLOTS")?.unwrap_or(DEFAULT_HEAD_LAG_SLOTS);
        RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::confirmed())
            .get_slot()
            .await
            .map(|slot| slot.saturating_sub(head_lag_slots))
            .map_err(err)?
    } else {
        return Err(err(
            "BLOCK_CRAWLER_START_SLOT is required when BLOCK_CRAWLER_END_SLOT is set",
        ));
    };

    Ok(RpcBlockCrawler::new(
        rpc_url,
        start_slot,
        end_slot,
        None,
        RpcBlockConfig {
            encoding: Some(UiTransactionEncoding::Binary),
            max_supported_transaction_version: Some(0),
            ..Default::default()
        },
        Some(1),
        None,
    ))
}

fn clickhouse_config(config: ClickHouseConfig) -> ClickHouseConfig {
    if enabled("CLICKHOUSE_ASYNC_INSERT") {
        config.with_insert_settings(ClickHouseInsertSettings::AsyncWait(
            ClickHouseAsyncInsertSettings::default(),
        ))
    } else {
        config
    }
}

fn prometheus_metrics() -> CarbonResult<PrometheusMetrics> {
    let addr = env::var("PROMETHEUS_METRICS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9464".to_string())
        .parse::<SocketAddr>()
        .map_err(err)?;
    Ok(PrometheusMetrics::with_server(
        PrometheusServerConfig::new().listen_addr(addr),
    ))
}

fn enabled(name: &str) -> bool {
    matches!(
        env::var(name)
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "y" | "on"
    )
}

fn init_logger() {
    let mut logger = env_logger::Builder::new();
    logger.filter_level(log::LevelFilter::Debug);
    if let Ok(log_level) = env::var("LOG_LEVEL") {
        logger.parse_filters(&log_level);
    }
    logger.init();
}

fn required_env(name: &str) -> CarbonResult<String> {
    env::var(name).map_err(err)
}

fn optional_env_u64(name: &str) -> CarbonResult<Option<u64>> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|error| err(format!("Invalid {name}={value:?}: {error}")))
        })
        .transpose()
}

fn err(error: impl std::fmt::Display) -> CarbonError {
    CarbonError::Custom(error.to_string())
}
