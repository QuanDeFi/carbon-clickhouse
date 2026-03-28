use std::{env, sync::Arc};

use solana_transaction_status::UiTransactionEncoding;

use {
    async_trait::async_trait,
    carbon_core::{
        error::CarbonResult, instruction::InstructionProcessorInputType,
        metrics::MetricsCollection, processor::Processor,
    },
    carbon_jupiter_swap_decoder::{
        instructions::{CpiEvent, JupiterSwapInstruction},
        JupiterSwapDecoder,
    },
    carbon_rpc_block_crawler_datasource::{RpcBlockConfig, RpcBlockCrawler},
    clap::Parser,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    start_slot: u64,

    #[arg(short, long)]
    end_slot: u64,
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
    let max_concurrent_requests = env_usize("BLOCK_CRAWLER_MAX_CONCURRENT_REQUESTS", 1);
    let channel_buffer_size = env_usize("BLOCK_CRAWLER_CHANNEL_BUFFER_SIZE", 10);

    let rpc_block_ds = RpcBlockCrawler::new(
        env::var("RPC_URL").unwrap_or_default(),
        args.start_slot,
        Some(args.end_slot),
        None,
        RpcBlockConfig {
            encoding: Some(UiTransactionEncoding::Binary),
            max_supported_transaction_version: Some(0),
            ..Default::default()
        },
        Some(max_concurrent_requests),
        Some(channel_buffer_size),
    );

    carbon_core::pipeline::Pipeline::builder()
        .datasource(rpc_block_ds)
        .instruction(JupiterSwapDecoder, JupiterSwapInstructionProcessor)
        .shutdown_strategy(carbon_core::pipeline::ShutdownStrategy::Immediate)
        .build()?
        .run()
        .await?;

    Ok(())
}
pub struct JupiterSwapInstructionProcessor;

#[async_trait]
impl Processor for JupiterSwapInstructionProcessor {
    type InputType = InstructionProcessorInputType<JupiterSwapInstruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (metadata, instruction, _nested_instructions, _) = data;

        if let JupiterSwapInstruction::CpiEvent(cpi_event) = instruction.data {
            match *cpi_event {
                CpiEvent::FeeEvent(fee_event) => {
                    log::info!(
                        "Jupiter fee event: {fee_event:#?} on slot {}",
                        metadata.transaction_metadata.slot
                    );
                }
                CpiEvent::SwapEvent(swap_event) => {
                    log::info!(
                        "Jupiter swap event: {swap_event:#?} on slot {}",
                        metadata.transaction_metadata.slot
                    );
                }
                CpiEvent::SwapsEvent(swaps_event) => {
                    log::info!(
                        "Jupiter swaps event: {swaps_event:#?} on slot {}",
                        metadata.transaction_metadata.slot
                    );
                }
                _ => {}
            }
        }

        Ok(())
    }
}
