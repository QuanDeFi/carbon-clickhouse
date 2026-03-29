use std::{env, sync::Arc};

use solana_transaction_status::UiTransactionEncoding;

use {
    carbon_core::{
        error::{CarbonResult, Error as CarbonError},
        pipeline::ShutdownStrategy,
    },
    carbon_jupiter_swap_decoder::{instructions::clickhouse::setup_clickhouse, JupiterSwapDecoder},
    carbon_log_metrics::LogMetrics,
    carbon_rpc_block_crawler_datasource::{RpcBlockConfig, RpcBlockCrawler},
    clap::Parser,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    start_slot: Option<u64>,

    #[arg(short, long)]
    end_slot: Option<u64>,
}

fn env_usize(name: &str, default: usize) -> usize {
    match env::var(name) {
        Ok(value) => match value.parse::<usize>() {
            Ok(parsed) => parsed,
            Err(err) => {
                log::warn!("Invalid {name}={value:?}: {err}. Using default {default}.");
                default
            }
        },
        Err(_) => default,
    }
}

fn env_u64(name: &str) -> CarbonResult<u64> {
    let value =
        env::var(name).map_err(|err| CarbonError::Custom(format!("{name} must be set ({err})")))?;

    value
        .parse::<u64>()
        .map_err(|err| CarbonError::Custom(format!("Invalid {name}={value:?}: {err}")))
}

#[tokio::main]
pub async fn main() -> CarbonResult<()> {
    dotenv::dotenv().ok();
    let mut logger = env_logger::Builder::new();
    logger.filter_level(log::LevelFilter::Info);
    if let Ok(log_level) = env::var("LOG_LEVEL") {
        logger.parse_filters(&log_level);
    }
    logger.init();

    let args = Args::parse();
    let start_slot = match args.start_slot {
        Some(start_slot) => start_slot,
        None => env_u64("BLOCK_CRAWLER_START_SLOT")?,
    };
    let end_slot = args.end_slot.or_else(|| {
        env::var("BLOCK_CRAWLER_END_SLOT")
            .ok()
            .and_then(|value| value.parse().ok())
    });

    let database_url = env::var("DATABASE_URL")
        .map_err(|err| CarbonError::Custom(format!("DATABASE_URL must be set ({err})")))?;
    let processor = setup_clickhouse(&database_url).await?;

    let rpc_block_ds = RpcBlockCrawler::new(
        env::var("RPC_URL").unwrap_or_default(),
        start_slot,
        end_slot,
        None,
        RpcBlockConfig {
            encoding: Some(UiTransactionEncoding::Binary),
            max_supported_transaction_version: Some(0),
            ..Default::default()
        },
        Some(env_usize("BLOCK_CRAWLER_MAX_CONCURRENT_REQUESTS", 1)),
        Some(env_usize("BLOCK_CRAWLER_CHANNEL_BUFFER_SIZE", 10)),
    );

    carbon_core::pipeline::Pipeline::builder()
        .datasource(rpc_block_ds)
        .metrics(Arc::new(LogMetrics::new_with_flush_interval(3)))
        .instruction(JupiterSwapDecoder, processor)
        .shutdown_strategy(ShutdownStrategy::Immediate)
        .build()?
        .run()
        .await?;

    Ok(())
}
