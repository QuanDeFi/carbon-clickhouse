use std::{env, net::SocketAddr, sync::Arc};

use solana_transaction_status::UiTransactionEncoding;

use {
    carbon_core::{
        error::{CarbonResult, Error as CarbonError},
        pipeline::ShutdownStrategy,
    },
    carbon_jupiter_swap_decoder::{
        instructions::clickhouse::{bootstrap_clickhouse_from_database_url, clickhouse_processor},
        JupiterSwapDecoder,
    },
    carbon_log_metrics::LogMetrics,
    carbon_prometheus_metrics::{PrometheusMetrics, PrometheusServerConfig},
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

fn prometheus_metrics() -> CarbonResult<PrometheusMetrics> {
    let listen_addr = env::var("PROMETHEUS_METRICS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9464".to_string())
        .parse::<SocketAddr>()
        .map_err(|err| CarbonError::Custom(format!("Invalid PROMETHEUS_METRICS_ADDR: {err}")))?;

    Ok(PrometheusMetrics::with_server(
        PrometheusServerConfig::new().listen_addr(listen_addr),
    ))
}

fn optional_env_u64(name: &str) -> CarbonResult<Option<u64>> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse::<u64>()
                .map_err(|err| CarbonError::Custom(format!("Invalid {name}={value:?}: {err}")))
        })
        .transpose()
}

fn optional_env_bool(name: &str) -> CarbonResult<Option<bool>> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_bool(name, &value))
        .transpose()
}

fn env_bool(name: &str, default: bool) -> CarbonResult<bool> {
    match env::var(name) {
        Ok(value) if !value.trim().is_empty() => parse_bool(name, &value),
        _ => Ok(default),
    }
}

fn parse_bool(name: &str, value: &str) -> CarbonResult<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Ok(true),
        "0" | "false" | "no" | "n" | "off" => Ok(false),
        _ => Err(CarbonError::Custom(format!(
            "Invalid {name}={value:?}: expected true/false"
        ))),
    }
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
    let mut clickhouse_config = bootstrap_clickhouse_from_database_url(&database_url).await?;
    if env_bool("CLICKHOUSE_ASYNC_INSERT", false)? {
        clickhouse_config = clickhouse_config.with_insert_settings(
            carbon_core::clickhouse::ClickHouseInsertSettings::AsyncWait(
                carbon_core::clickhouse::ClickHouseAsyncInsertSettings {
                    busy_timeout_ms: optional_env_u64("CLICKHOUSE_ASYNC_INSERT_BUSY_TIMEOUT_MS")?,
                    max_data_size: optional_env_u64("CLICKHOUSE_ASYNC_INSERT_MAX_DATA_SIZE")?,
                    max_query_number: optional_env_u64("CLICKHOUSE_ASYNC_INSERT_MAX_QUERY_NUMBER")?,
                    deduplicate: optional_env_bool("CLICKHOUSE_ASYNC_INSERT_DEDUPLICATE")?,
                },
            ),
        );
    }
    let instruction_processor = clickhouse_processor(clickhouse_config);

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
        .metrics(Arc::new(prometheus_metrics()?))
        .instruction(JupiterSwapDecoder, instruction_processor)
        .shutdown_strategy(ShutdownStrategy::Immediate)
        .build()?
        .run()
        .await?;

    Ok(())
}
