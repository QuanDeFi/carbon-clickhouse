use std::{env, net::SocketAddr, sync::Arc};

use {
    carbon_core::{
        datasource::Datasource,
        error::{CarbonResult, Error as CarbonError},
        pipeline::{Pipeline, ShutdownStrategy},
    },
    carbon_helius_gpa_v2_datasource::{HeliusGpaV2Config, HeliusGpaV2Datasource},
    carbon_log_metrics::LogMetrics,
    carbon_prometheus_metrics::{PrometheusMetrics, PrometheusServerConfig},
    carbon_rpc_gpa_datasource::GpaDatasource,
    carbon_token_program_decoder::{
        accounts::clickhouse::{
            bootstrap_clickhouse_from_database_url, clickhouse_processor,
            TokenProgramClickHouseAccountProcessor,
        },
        TokenProgramDecoder, PROGRAM_ID as TOKEN_PROGRAM_ID,
    },
    clap::{Parser, ValueEnum},
    solana_account_decoder::UiAccountEncoding,
    solana_client::{
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
        rpc_filter::{Memcmp, RpcFilterType},
    },
    solana_commitment_config::CommitmentConfig,
    solana_pubkey::Pubkey,
};

const TOKEN_ACCOUNT_SIZE: u64 = 165;
const TOKEN_ACCOUNT_MINT_OFFSET: usize = 0;
const TOKEN_ACCOUNT_OWNER_OFFSET: usize = 32;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, value_enum, default_value_t = Source::HeliusGpaV2)]
    source: Source,

    #[arg(long)]
    token_owner: Option<String>,

    #[arg(long)]
    token_mint: Option<String>,

    #[arg(long)]
    helius_changed_since_slot: Option<u64>,

    #[arg(long)]
    helius_page_limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Source {
    Rpc,
    HeliusGpaV2,
}

#[tokio::main]
pub async fn main() -> CarbonResult<()> {
    dotenv::dotenv().ok();
    init_logger();

    let args = Args::parse();
    let account_filter = token_account_filter(&args)?;
    let database_url = required_env("DATABASE_URL")?;

    let mut config = bootstrap_clickhouse_from_database_url(&database_url).await?;
    if env_bool("CLICKHOUSE_ASYNC_INSERT", false)? {
        config = config.with_insert_settings(
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
    let processor = clickhouse_processor(config);

    match args.source {
        Source::Rpc => {
            let datasource = GpaDatasource::new_with_config(
                required_env("RPC_URL")?,
                TOKEN_PROGRAM_ID,
                account_filter,
            );
            run_pipeline(datasource, processor).await
        }
        Source::HeliusGpaV2 => {
            let datasource = HeliusGpaV2Datasource::new_with_config(
                required_env("HELIUS_RPC_URL")?,
                TOKEN_PROGRAM_ID,
                HeliusGpaV2Config::new(
                    Some(account_filter),
                    args.helius_changed_since_slot
                        .or(optional_env_u64("HELIUS_GPA_V2_CHANGED_SINCE_SLOT")?),
                    args.helius_page_limit
                        .or(optional_env_u32("HELIUS_GPA_V2_PAGE_LIMIT")?)
                        .or(Some(1000)),
                ),
            );
            run_pipeline(datasource, processor).await
        }
    }?;

    log::info!("token account snapshot complete");
    Ok(())
}

async fn run_pipeline<D>(
    datasource: D,
    processor: TokenProgramClickHouseAccountProcessor,
) -> CarbonResult<()>
where
    D: Datasource + 'static,
{
    Pipeline::builder()
        .datasource(datasource)
        .metrics(Arc::new(LogMetrics::new_with_flush_interval(3)))
        .metrics(Arc::new(prometheus_metrics()?))
        .account(TokenProgramDecoder, processor)
        .shutdown_strategy(ShutdownStrategy::ProcessPending)
        .build()?
        .run()
        .await
}

fn token_account_filter(args: &Args) -> CarbonResult<RpcProgramAccountsConfig> {
    let token_owner = args
        .token_owner
        .clone()
        .or_else(|| env::var("TOKEN_ACCOUNT_OWNER").ok())
        .map(|value| parse_pubkey("TOKEN_ACCOUNT_OWNER", &value))
        .transpose()?;
    let token_mint = args
        .token_mint
        .clone()
        .or_else(|| env::var("TOKEN_MINT").ok())
        .map(|value| parse_pubkey("TOKEN_MINT", &value))
        .transpose()?;

    if token_owner.is_none() && token_mint.is_none() {
        return Err(CarbonError::Custom(
            "Set TOKEN_ACCOUNT_OWNER, TOKEN_MINT, --token-owner, or --token-mint to keep GPA bounded"
                .to_string(),
        ));
    }

    let mut filters = vec![RpcFilterType::DataSize(TOKEN_ACCOUNT_SIZE)];
    if let Some(token_mint) = token_mint {
        filters.push(RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
            TOKEN_ACCOUNT_MINT_OFFSET,
            token_mint.as_ref(),
        )));
    }
    if let Some(token_owner) = token_owner {
        filters.push(RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
            TOKEN_ACCOUNT_OWNER_OFFSET,
            token_owner.as_ref(),
        )));
    }

    Ok(RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::confirmed()),
            ..Default::default()
        },
        with_context: Some(true),
        ..Default::default()
    })
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

fn prometheus_metrics() -> CarbonResult<PrometheusMetrics> {
    let listen_addr = env::var("PROMETHEUS_METRICS_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:9464".to_string())
        .parse::<SocketAddr>()
        .map_err(|err| CarbonError::Custom(format!("Invalid PROMETHEUS_METRICS_ADDR: {err}")))?;

    Ok(PrometheusMetrics::with_server(
        PrometheusServerConfig::new().listen_addr(listen_addr),
    ))
}

fn parse_pubkey(name: &str, value: &str) -> CarbonResult<Pubkey> {
    value
        .parse()
        .map_err(|err| CarbonError::Custom(format!("Invalid {name}={value:?}: {err}")))
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

fn optional_env_u32(name: &str) -> CarbonResult<Option<u32>> {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .parse::<u32>()
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
