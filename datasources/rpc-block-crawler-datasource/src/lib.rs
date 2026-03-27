use carbon_core::datasource::DatasourceId;
pub use solana_client::rpc_config::RpcBlockConfig;
use solana_hash::Hash;
use std::{collections::HashSet, str::FromStr};
use {
    async_trait::async_trait,
    carbon_core::{
        datasource::{Datasource, TransactionUpdate, Update, UpdateType},
        error::CarbonResult,
        metrics::MetricsCollection,
        transformers::transaction_metadata_from_original_meta,
    },
    futures::StreamExt,
    solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::SerializableTransaction},
    solana_commitment_config::CommitmentConfig,
    solana_transaction_status::{UiConfirmedBlock, UiTransactionEncoding},
    std::{
        sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        },
        time::{Duration, Instant},
    },
    tokio::{
        sync::mpsc::{self, error::TrySendError, Receiver, Sender},
        sync::Mutex,
        task::JoinHandle,
    },
    tokio_util::sync::CancellationToken,
};

const CHANNEL_BUFFER_SIZE: usize = 1000;
const MAX_CONCURRENT_REQUESTS: usize = 10;
const BLOCK_INTERVAL: Duration = Duration::from_millis(100);
const MAX_RETRY_ATTEMPTS_PER_SLOT: usize = 8;
const ADAPTIVE_RATE_LIMIT_RPS_ON_LIMIT: u64 = 5;
const GET_BLOCK_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const HEAD_SLOT_POLL_INTERVAL: Duration = Duration::from_secs(1);
const TAIL_START_SLOT_OFFSET: u64 = 5;

const POLL_LATEST_SLOT: &str = "stage.poll_latest_slot.slot";
const SCHEDULE_SLOT: &str = "stage.schedule_slot.slot";
const FETCH_BLOCK_GET_BLOCK: &str = "stage.fetch_block_get_block.slot";
const FETCH_BLOCK_GET_BLOCK_EXIT: &str = "stage.fetch_block_get_block.exit.slot";
const RETRY_BLOCK_FETCH: &str = "stage.retry_block_fetch.slot";
const CLASSIFY_BLOCK_FETCH_RESULT: &str = "stage.classify_block_fetch_result.slot";
const SKIP_SLOT_SKIPPABLE_RPC_ERROR: &str = "stage.skip_slot_skippable_rpc_error.slot";
const ENQUEUE_FETCHED_BLOCK: &str = "stage.enqueue_fetched_block.slot";
const DEQUEUE_FETCHED_BLOCK: &str = "stage.dequeue_fetched_block.slot";
const EXTRACT_TRANSACTIONS_FROM_BLOCK: &str = "stage.extract_transactions_from_block.slot";
const EXTRACT_TRANSACTIONS_FROM_BLOCK_EXIT: &str =
    "stage.extract_transactions_from_block.exit.slot";
const DECODE_TRANSACTION: &str = "stage.decode_transaction.slot";
const SKIP_TRANSACTION_DECODE_OR_META_ERROR: &str =
    "stage.skip_transaction_decode_or_meta_error.slot";
const TRANSFORM_TRANSACTION_META: &str = "stage.transform_transaction_meta.slot";
const SKIP_TRANSACTION_FAILED_STATUS: &str = "stage.skip_transaction_failed_status.slot";
const BUILD_TRANSACTION_UPDATE: &str = "stage.build_transaction_update.slot";
const ENQUEUE_PIPELINE_UPDATE: &str = "stage.enqueue_pipeline_update.slot";
const ENQUEUE_PIPELINE_UPDATE_EXIT: &str = "stage.enqueue_pipeline_update.exit.slot";
const DROP_OR_DELAY_ON_CHANNEL_BACKPRESSURE: &str =
    "stage.drop_or_delay_on_channel_backpressure.slot";
const DEQUEUE_PIPELINE_UPDATE: &str = "stage.dequeue_pipeline_update.slot";
const BUILD_CORE_TRANSACTION_METADATA: &str = "stage.build_core_transaction_metadata.slot";
const EXTRACT_INSTRUCTIONS_WITH_METADATA: &str = "stage.extract_instructions_with_metadata.slot";
const NEST_INSTRUCTIONS: &str = "stage.nest_instructions.slot";
const APPLY_INSTRUCTION_FILTERS: &str = "stage.apply_instruction_filters.slot";
const DECODE_INSTRUCTION_JUPITER_SWAP_DECODER: &str =
    "stage.decode_instruction_jupiter_swap_decoder.slot";
const PROCESS_INSTRUCTION_POSTGRES_INSTRUCTION_PROCESSOR: &str =
    "stage.process_instruction_postgres_instruction_processor.slot";
const BUILD_INSTRUCTION_ROW_WRAPPER: &str = "stage.build_instruction_row_wrapper.slot";
const UPSERT_INSTRUCTION_ROW_POSTGRES: &str = "stage.upsert_instruction_row_postgres.slot";

pub const STAGE_SLOT_METRICS: [&str; 25] = [
    POLL_LATEST_SLOT,
    SCHEDULE_SLOT,
    FETCH_BLOCK_GET_BLOCK,
    RETRY_BLOCK_FETCH,
    CLASSIFY_BLOCK_FETCH_RESULT,
    SKIP_SLOT_SKIPPABLE_RPC_ERROR,
    ENQUEUE_FETCHED_BLOCK,
    DEQUEUE_FETCHED_BLOCK,
    EXTRACT_TRANSACTIONS_FROM_BLOCK,
    DECODE_TRANSACTION,
    SKIP_TRANSACTION_DECODE_OR_META_ERROR,
    TRANSFORM_TRANSACTION_META,
    SKIP_TRANSACTION_FAILED_STATUS,
    BUILD_TRANSACTION_UPDATE,
    ENQUEUE_PIPELINE_UPDATE,
    DROP_OR_DELAY_ON_CHANNEL_BACKPRESSURE,
    DEQUEUE_PIPELINE_UPDATE,
    BUILD_CORE_TRANSACTION_METADATA,
    EXTRACT_INSTRUCTIONS_WITH_METADATA,
    NEST_INSTRUCTIONS,
    APPLY_INSTRUCTION_FILTERS,
    DECODE_INSTRUCTION_JUPITER_SWAP_DECODER,
    PROCESS_INSTRUCTION_POSTGRES_INSTRUCTION_PROCESSOR,
    BUILD_INSTRUCTION_ROW_WRAPPER,
    UPSERT_INSTRUCTION_ROW_POSTGRES,
];

#[derive(Clone, Copy, Debug)]
enum RpcErrorAction {
    Skip,
    Retry,
    UserAction,
    StopPipeline,
    Unknown,
}

fn classify_rpc_error(error_string: &str) -> RpcErrorAction {
    let contains_http_code_token = |code: &str| {
        error_string
            .split(|c: char| !c.is_ascii_digit())
            .any(|token| token == code)
    };

    // Stop-pipeline: endpoint/auth/config issues where retrying slots is not useful.
    if contains_http_code_token("401")
        || contains_http_code_token("403")
        || contains_http_code_token("404")
        || error_string.contains("-32601")
    {
        return RpcErrorAction::StopPipeline;
    }

    // User action: request/build/config fixes needed.
    if error_string.contains("-32002")
        || error_string.contains("-32003")
        || error_string.contains("-32006")
        || error_string.contains("-32013")
        || error_string.contains("-32015")
        || error_string.contains("-32602")
    {
        return RpcErrorAction::UserAction;
    }

    // Skippable for block-crawler continuity (node/storage capability gaps or skipped slots).
    if error_string.contains("-32001")
        || error_string.contains("-32008")
        || error_string.contains("-32007")
        || error_string.contains("-32009")
        || error_string.contains("-32010")
        || error_string.contains("-32011")
    {
        return RpcErrorAction::Skip;
    }

    // Transient/retryable.
    if error_string.contains("-32004")
        || error_string.contains("-32005")
        || error_string.contains("-32014")
        || error_string.contains("-32016")
        || contains_http_code_token("429")
        || contains_http_code_token("503")
        || error_string.contains("timeout")
        || error_string.contains("timed out")
    {
        return RpcErrorAction::Retry;
    }

    RpcErrorAction::Unknown
}

fn contains_http_code_token(error_string: &str, code: &str) -> bool {
    error_string
        .split(|c: char| !c.is_ascii_digit())
        .any(|token| token == code)
}

fn is_rate_limit_error(error_string: &str) -> bool {
    contains_http_code_token(error_string, "429")
        || error_string
            .to_ascii_lowercase()
            .contains("too many requests")
}

fn throttle_millis_for_rps(rps: u64) -> u64 {
    1000u64.div_ceil(rps.max(1)).max(1)
}

fn adaptive_concurrency_limit(max_concurrent_requests: usize, lag: u64) -> usize {
    // Lag-aware startup behavior:
    // - Near head (tail mode and low lag): adaptive policy overrides configured max.
    // - Far from head (high lag): use configured max for catch-up throughput.
    if lag > 32 {
        return max_concurrent_requests.max(1);
    }

    if lag >= 8 {
        4
    } else if lag >= 2 {
        2
    } else {
        1
    }
}

fn normalize_block_config_for_decode(mut block_config: RpcBlockConfig) -> RpcBlockConfig {
    if matches!(
        block_config.encoding,
        None | Some(UiTransactionEncoding::Binary)
    ) {
        // Binary uses legacy base58 payloads and is slower to decode.
        block_config.encoding = Some(UiTransactionEncoding::Base64);
    }
    block_config
}

async fn update_stage_slot(metrics: &MetricsCollection, stage: &str, slot: u64) {
    metrics
        .update_gauge(stage, slot as f64)
        .await
        .unwrap_or_else(|value| log::error!("Error recording metric: {value}"));
}

async fn wait_for_global_request_turn(
    request_gate: &Arc<Mutex<Instant>>,
    throttle_ms: &std::sync::atomic::AtomicU64,
) {
    let interval_ms = throttle_ms.load(Ordering::Relaxed);
    if interval_ms == 0 {
        return;
    }

    let interval = Duration::from_millis(interval_ms);
    let mut last_request_at = request_gate.lock().await;
    let now = Instant::now();
    let earliest_next = *last_request_at + interval;
    if earliest_next > now {
        tokio::time::sleep(earliest_next - now).await;
    }
    *last_request_at = Instant::now();
}

/// RpcBlockCrawler is a datasource that crawls the Solana blockchain for blocks and sends them to the sender.
/// It uses a channel to send blocks to the task processor.
pub struct RpcBlockCrawler {
    pub rpc_url: String,
    pub start_slot: Option<u64>,
    pub end_slot: Option<u64>,
    pub mode_label: String,
    pub block_interval: Duration,
    pub block_config: RpcBlockConfig,
    pub max_concurrent_requests: usize,
    pub channel_buffer_size: usize,
    pub request_throttle: Option<Duration>,
}

impl RpcBlockCrawler {
    pub fn new(
        rpc_url: String,
        start_slot: Option<u64>,
        end_slot: Option<u64>,
        block_interval: Option<Duration>,
        block_config: RpcBlockConfig,
        max_concurrent_requests: Option<usize>,
        channel_buffer_size: Option<usize>,
    ) -> Self {
        Self {
            rpc_url,
            start_slot,
            end_slot,
            mode_label: "unspecified".to_string(),
            block_config,
            block_interval: block_interval.unwrap_or(BLOCK_INTERVAL),
            max_concurrent_requests: max_concurrent_requests.unwrap_or(MAX_CONCURRENT_REQUESTS),
            channel_buffer_size: channel_buffer_size.unwrap_or(CHANNEL_BUFFER_SIZE),
            request_throttle: None,
        }
    }
}

#[async_trait]
impl Datasource for RpcBlockCrawler {
    async fn consume(
        &self,
        id: DatasourceId,
        sender: Sender<(Update, DatasourceId)>,
        cancellation_token: CancellationToken,
        metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            self.rpc_url.clone(),
            self.block_config
                .commitment
                .unwrap_or(CommitmentConfig::finalized()),
        ));
        let (block_sender, block_receiver) = mpsc::channel(self.channel_buffer_size);

        let start_slot = match self.start_slot {
            Some(start_slot) => start_slot,
            None => {
                log::info!(
                    "RpcBlockCrawler start_slot not provided; defaulting to latest slot - {}",
                    TAIL_START_SLOT_OFFSET
                );
                let latest_slot = rpc_client.get_slot().await.map_err(|err| {
                    carbon_core::error::Error::FailedToConsumeDatasource(format!(
                        "Failed to determine latest slot: {err}"
                    ))
                })?;
                latest_slot.saturating_sub(TAIL_START_SLOT_OFFSET)
            }
        };

        let block_config = normalize_block_config_for_decode(self.block_config);
        if block_config.encoding != self.block_config.encoding {
            log::info!(
                "RpcBlockCrawler normalized getBlock encoding from {:?} to {:?} for decode efficiency",
                self.block_config.encoding,
                block_config.encoding
            );
        }
        log::info!(
            "RpcBlockCrawler mode={} start_slot={} end_slot={:?} interval_ms={} max_concurrent_requests={} request_throttle_ms={}",
            self.mode_label,
            start_slot,
            self.end_slot,
            self.block_interval.as_millis(),
            self.max_concurrent_requests,
            self.request_throttle.map(|d| d.as_millis()).unwrap_or(0)
        );

        let block_fetcher = block_fetcher(
            rpc_client,
            start_slot,
            self.end_slot,
            self.block_interval,
            block_config,
            block_sender,
            self.max_concurrent_requests,
            cancellation_token.clone(),
            metrics.clone(),
            self.request_throttle,
        );

        let task_processor = task_processor(
            block_receiver,
            sender,
            id,
            cancellation_token.clone(),
            metrics.clone(),
        );

        tokio::spawn(async move {
            tokio::select! {
                _ = block_fetcher => {},
                _ = task_processor => {},
            }
        });

        Ok(())
    }

    fn update_types(&self) -> Vec<UpdateType> {
        vec![UpdateType::Transaction]
    }
}

#[allow(clippy::too_many_arguments)]
fn block_fetcher(
    rpc_client: Arc<RpcClient>,
    start_slot: u64,
    end_slot: Option<u64>,
    block_interval: Duration,
    block_config: RpcBlockConfig,
    block_sender: Sender<(u64, UiConfirmedBlock)>,
    max_concurrent_requests: usize,
    cancellation_token: CancellationToken,
    metrics: Arc<MetricsCollection>,
    request_throttle: Option<Duration>,
) -> JoinHandle<()> {
    let rpc_client_clone = rpc_client.clone();
    let in_flight_requests = Arc::new(AtomicUsize::new(0));
    let request_gate = Arc::new(Mutex::new(Instant::now() - Duration::from_secs(60)));
    let adaptive_throttle_ms = Arc::new(std::sync::atomic::AtomicU64::new(
        request_throttle
            .map(|duration| duration.as_millis() as u64)
            .unwrap_or(0),
    ));
    let cancellation_token_for_stream = cancellation_token.clone();
    let cancellation_token_for_requests = cancellation_token.clone();

    tokio::spawn(async move {
        let fetch_stream_task = async {
            let metrics_for_stream = metrics.clone();
            let in_flight_requests_for_stream = Arc::clone(&in_flight_requests);
            let request_gate_for_stream = Arc::clone(&request_gate);
            let adaptive_throttle_for_stream = Arc::clone(&adaptive_throttle_ms);
            let fetch_stream = async_stream::stream! {
                let mut current_slot = start_slot;
                let mut latest_slot = current_slot;
                let mut lag_to_head_known = end_slot.is_some();
                let mut next_head_poll_at = Instant::now();
                loop {
                    if let Some(end) = end_slot {
                        latest_slot = end;
                        if current_slot > end {
                            break;
                        }
                    } else {
                        // No getBlock work is scheduled until we successfully establish lag to head.
                        if !lag_to_head_known || current_slot >= latest_slot {
                            let now = Instant::now();
                            if now < next_head_poll_at {
                                tokio::time::sleep(next_head_poll_at - now).await;
                            }
                            wait_for_global_request_turn(
                                &request_gate_for_stream,
                                &adaptive_throttle_for_stream,
                            )
                            .await;
                            next_head_poll_at = Instant::now() + HEAD_SLOT_POLL_INTERVAL;
                            match rpc_client_clone.get_slot().await {
                                Ok(slot) => {
                                    update_stage_slot(
                                        &metrics_for_stream,
                                        POLL_LATEST_SLOT,
                                        slot,
                                    )
                                    .await;
                                    latest_slot = slot;
                                    lag_to_head_known = true;
                                    if current_slot > latest_slot {
                                        log::debug!(
                                            "Waiting for new blocks... Current: {current_slot}, Latest: {latest_slot}"
                                        );
                                        tokio::time::sleep(block_interval).await;
                                        continue;
                                    }
                                }
                                Err(e) => {
                                    log::error!("Error fetching latest slot: {e:?}");
                                    tokio::time::sleep(block_interval).await;
                                    continue;
                                }
                            }
                        }
                        if !lag_to_head_known {
                            tokio::time::sleep(block_interval).await;
                            continue;
                        }
                        if latest_slot.saturating_sub(current_slot) > 100 {
                            log::debug!(
                                "Current slot {} is behind latest slot {} by {}",
                                current_slot,
                                latest_slot,
                                latest_slot.saturating_sub(current_slot)
                            );
                        }
                    }

                    let lag = latest_slot.saturating_sub(current_slot);
                    let adaptive_limit = adaptive_concurrency_limit(max_concurrent_requests, lag);
                    metrics_for_stream
                        .update_gauge(
                            "block_crawler_adaptive_concurrency_limit",
                            adaptive_limit as f64,
                        )
                        .await
                        .unwrap_or_else(|value| {
                            log::error!("Error recording metric: {value}")
                        });
                    while in_flight_requests_for_stream.load(Ordering::Relaxed) >= adaptive_limit {
                        if cancellation_token_for_stream.is_cancelled() {
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(10)).await;
                    }

                    update_stage_slot(&metrics_for_stream, SCHEDULE_SLOT, current_slot).await;
                    yield current_slot;
                    current_slot += 1;
                }
            };

            let metrics_for_fetch = metrics.clone();
            let metrics_for_enqueue = metrics.clone();
            let block_sender_for_enqueue = block_sender.clone();

            fetch_stream
                .map(move |slot| {
                    let rpc_client = Arc::clone(&rpc_client);
                    let metrics = metrics_for_fetch.clone();
                    let in_flight_requests = Arc::clone(&in_flight_requests);
                    let request_gate = Arc::clone(&request_gate);
                    let adaptive_throttle_ms = Arc::clone(&adaptive_throttle_ms);
                    let block_config = block_config.clone();
                    let cancellation_token_for_requests = cancellation_token_for_requests.clone();

                    async move {
                        let mut attempt = 0usize;
                        let result = loop {
                            if cancellation_token_for_requests.is_cancelled() {
                                break None;
                            }
                            attempt += 1;
                            wait_for_global_request_turn(&request_gate, &adaptive_throttle_ms)
                                .await;
                            update_stage_slot(&metrics, FETCH_BLOCK_GET_BLOCK, slot).await;
                            // Hold fetch resource only for the actual RPC call.
                            in_flight_requests.fetch_add(1, Ordering::Relaxed);
                            let start = Instant::now();
                            match tokio::time::timeout(
                                GET_BLOCK_REQUEST_TIMEOUT,
                                rpc_client.get_block_with_config(slot, block_config),
                            )
                            .await
                            {
                                Err(_) => {
                                    let error_string = format!(
                                        "getBlock request timed out after {} ms",
                                        GET_BLOCK_REQUEST_TIMEOUT.as_millis()
                                    );
                                    if is_rate_limit_error(&error_string) {
                                        let throttle_ms = throttle_millis_for_rps(
                                            ADAPTIVE_RATE_LIMIT_RPS_ON_LIMIT,
                                        );
                                        let previous = adaptive_throttle_ms
                                            .fetch_max(throttle_ms, Ordering::Relaxed);
                                        if previous < throttle_ms {
                                            log::warn!(
                                                "Adaptive throttle enabled after rate-limit error: {} req/s ({} ms/request)",
                                                ADAPTIVE_RATE_LIMIT_RPS_ON_LIMIT,
                                                throttle_ms
                                            );
                                            metrics
                                                .increment_counter(
                                                    "block_crawler_adaptive_throttle_activations",
                                                    1,
                                                )
                                                .await
                                                .unwrap_or_else(|value| {
                                                    log::error!(
                                                        "Error recording metric: {value}"
                                                    )
                                                });
                                        }
                                    }
                                    update_stage_slot(&metrics, CLASSIFY_BLOCK_FETCH_RESULT, slot)
                                        .await;
                                    update_stage_slot(&metrics, RETRY_BLOCK_FETCH, slot).await;
                                    update_stage_slot(&metrics, FETCH_BLOCK_GET_BLOCK_EXIT, slot)
                                        .await;
                                    metrics
                                        .increment_counter("block_crawler_retryable_errors", 1)
                                        .await
                                        .unwrap_or_else(|value| {
                                            log::error!("Error recording metric: {value}")
                                        });
                                    if attempt >= MAX_RETRY_ATTEMPTS_PER_SLOT {
                                        log::warn!(
                                            "Retry budget exhausted for slot {slot} after {attempt} attempts; last error: {error_string}"
                                        );
                                        metrics
                                            .increment_counter(
                                                "block_crawler_blocks_failed_terminal",
                                                1,
                                            )
                                            .await
                                            .unwrap_or_else(|value| {
                                                log::error!("Error recording metric: {value}")
                                            });
                                        in_flight_requests.fetch_sub(1, Ordering::Relaxed);
                                        break None;
                                    }
                                    in_flight_requests.fetch_sub(1, Ordering::Relaxed);
                                    log::debug!(
                                        "Retrying slot {slot} attempt={attempt} due to retryable RPC error: {error_string}"
                                    );
                                    continue;
                                }
                                Ok(Ok(block)) => {
                                    update_stage_slot(&metrics, CLASSIFY_BLOCK_FETCH_RESULT, slot).await;
                                    update_stage_slot(&metrics, FETCH_BLOCK_GET_BLOCK_EXIT, slot)
                                        .await;
                                    in_flight_requests.fetch_sub(1, Ordering::Relaxed);
                                    let time_taken = start.elapsed().as_millis();
                                    metrics
                                        .record_histogram(
                                            "block_crawler_blocks_fetch_times_milliseconds",
                                            time_taken as f64,
                                        )
                                        .await
                                        .unwrap_or_else(|value| {
                                            log::error!("Error recording metric: {value}")
                                        });

                                    metrics
                                        .increment_counter("block_crawler_blocks_fetched", 1)
                                        .await
                                        .unwrap_or_else(|value| {
                                            log::error!("Error recording metric: {value}")
                                        });

                                    break Some((slot, block));
                                }
                                Ok(Err(e)) => {
                                    let error_string = e.to_string();
                                    match classify_rpc_error(&error_string) {
                                        RpcErrorAction::Skip => {
                                            update_stage_slot(
                                                &metrics,
                                                CLASSIFY_BLOCK_FETCH_RESULT,
                                                slot,
                                            )
                                            .await;
                                            log::debug!(
                                                "Skipping slot {slot} due to skippable RPC error: {error_string}"
                                            );
                                            update_stage_slot(
                                                &metrics,
                                                SKIP_SLOT_SKIPPABLE_RPC_ERROR,
                                                slot,
                                            )
                                            .await;
                                            update_stage_slot(
                                                &metrics,
                                                FETCH_BLOCK_GET_BLOCK_EXIT,
                                                slot,
                                            )
                                            .await;
                                            in_flight_requests.fetch_sub(1, Ordering::Relaxed);
                                            metrics
                                                .increment_counter("block_crawler_blocks_skipped", 1)
                                                .await
                                                .unwrap_or_else(|value| {
                                                    log::error!("Error recording metric: {value}")
                                                });
                                            break None;
                                        }
                                        RpcErrorAction::Retry => {
                                            if is_rate_limit_error(&error_string) {
                                                let throttle_ms =
                                                    throttle_millis_for_rps(ADAPTIVE_RATE_LIMIT_RPS_ON_LIMIT);
                                                let previous = adaptive_throttle_ms
                                                    .fetch_max(throttle_ms, Ordering::Relaxed);
                                                if previous < throttle_ms {
                                                    log::warn!(
                                                        "Adaptive throttle enabled after rate-limit error: {} req/s ({} ms/request)",
                                                        ADAPTIVE_RATE_LIMIT_RPS_ON_LIMIT,
                                                        throttle_ms
                                                    );
                                                    metrics
                                                        .increment_counter(
                                                            "block_crawler_adaptive_throttle_activations",
                                                            1,
                                                        )
                                                        .await
                                                        .unwrap_or_else(|value| {
                                                            log::error!(
                                                                "Error recording metric: {value}"
                                                            )
                                                        });
                                                }
                                            }
                                            update_stage_slot(
                                                &metrics,
                                                CLASSIFY_BLOCK_FETCH_RESULT,
                                                slot,
                                            )
                                            .await;
                                            update_stage_slot(&metrics, RETRY_BLOCK_FETCH, slot)
                                                .await;
                                            update_stage_slot(
                                                &metrics,
                                                FETCH_BLOCK_GET_BLOCK_EXIT,
                                                slot,
                                            )
                                            .await;
                                            metrics
                                                .increment_counter("block_crawler_retryable_errors", 1)
                                                .await
                                                .unwrap_or_else(|value| {
                                                    log::error!("Error recording metric: {value}")
                                                });
                                            if attempt >= MAX_RETRY_ATTEMPTS_PER_SLOT {
                                                log::warn!(
                                                    "Retry budget exhausted for slot {slot} after {attempt} attempts; last error: {error_string}"
                                                );
                                                metrics
                                                    .increment_counter(
                                                        "block_crawler_blocks_failed_terminal",
                                                        1,
                                                    )
                                                    .await
                                                    .unwrap_or_else(|value| {
                                                        log::error!("Error recording metric: {value}")
                                                    });
                                                in_flight_requests.fetch_sub(1, Ordering::Relaxed);
                                                break None;
                                            }
                                            in_flight_requests.fetch_sub(1, Ordering::Relaxed);
                                            log::debug!(
                                                "Retrying slot {slot} attempt={attempt} due to retryable RPC error: {error_string}"
                                            );
                                            continue;
                                        }
                                        RpcErrorAction::UserAction => {
                                            update_stage_slot(
                                                &metrics,
                                                CLASSIFY_BLOCK_FETCH_RESULT,
                                                slot,
                                            )
                                            .await;
                                            update_stage_slot(
                                                &metrics,
                                                FETCH_BLOCK_GET_BLOCK_EXIT,
                                                slot,
                                            )
                                            .await;
                                            in_flight_requests.fetch_sub(1, Ordering::Relaxed);
                                            log::error!(
                                                "User action required for slot {slot}: {error_string}"
                                            );
                                            metrics
                                                .increment_counter("block_crawler_user_action_errors", 1)
                                                .await
                                                .unwrap_or_else(|value| {
                                                    log::error!("Error recording metric: {value}")
                                                });
                                            break None;
                                        }
                                        RpcErrorAction::StopPipeline => {
                                            update_stage_slot(
                                                &metrics,
                                                CLASSIFY_BLOCK_FETCH_RESULT,
                                                slot,
                                            )
                                            .await;
                                            update_stage_slot(
                                                &metrics,
                                                FETCH_BLOCK_GET_BLOCK_EXIT,
                                                slot,
                                            )
                                            .await;
                                            in_flight_requests.fetch_sub(1, Ordering::Relaxed);
                                            log::error!(
                                                "Stopping pipeline due to fatal RPC error on slot {slot}: {error_string}"
                                            );
                                            metrics
                                                .increment_counter(
                                                    "block_crawler_stop_pipeline_errors",
                                                    1,
                                                )
                                                .await
                                                .unwrap_or_else(|value| {
                                                    log::error!("Error recording metric: {value}")
                                                });
                                            cancellation_token_for_requests.cancel();
                                            break None;
                                        }
                                        RpcErrorAction::Unknown => {
                                            update_stage_slot(
                                                &metrics,
                                                CLASSIFY_BLOCK_FETCH_RESULT,
                                                slot,
                                            )
                                            .await;
                                            update_stage_slot(
                                                &metrics,
                                                FETCH_BLOCK_GET_BLOCK_EXIT,
                                                slot,
                                            )
                                            .await;
                                            in_flight_requests.fetch_sub(1, Ordering::Relaxed);
                                            log::error!(
                                                "Unclassified RPC error fetching slot {slot}: {error_string}"
                                            );
                                            metrics
                                                .increment_counter(
                                                    "block_crawler_blocks_failed_terminal",
                                                    1,
                                                )
                                                .await
                                                .unwrap_or_else(|value| {
                                                    log::error!("Error recording metric: {value}")
                                                });
                                            break None;
                                        }
                                    }
                                }
                            }
                        };
                        result
                    }
                })
                .buffer_unordered(max_concurrent_requests)
                .for_each(move |result| {
                    let metrics = metrics_for_enqueue.clone();
                    let block_sender = block_sender_for_enqueue.clone();
                    async move {
                    if let Some((slot, block)) = result {
                        update_stage_slot(&metrics, ENQUEUE_FETCHED_BLOCK, slot).await;
                        if let Err(e) = block_sender.send((slot, block)).await {
                            log::error!("Failed to send block: {e:?}");
                        }
                    }
                }})
                .await;
        };

        tokio::select! {
            _ = cancellation_token.cancelled() => {
                log::info!("Cancelling RPC Crawler block fetcher...");
            }
            _ = fetch_stream_task => {}
        }
    })
}

/// Process the block and send the transactions to the sender
fn task_processor(
    block_receiver: Receiver<(u64, UiConfirmedBlock)>,
    sender: Sender<(Update, DatasourceId)>,
    id: DatasourceId,
    cancellation_token: CancellationToken,
    metrics: Arc<MetricsCollection>,
) -> JoinHandle<()> {
    let mut block_receiver = block_receiver;
    let sender = sender.clone();
    let id_for_loop = id.clone();

    tokio::spawn(async move {
        loop {
            tokio::select! {
            _ = cancellation_token.cancelled() => {
                log::info!("Cancelling RPC Crawler task processor...");
                break;
            }
            maybe_block = block_receiver.recv() => {
                match maybe_block {
                    Some((slot, block)) => {
                        update_stage_slot(&metrics, DEQUEUE_FETCHED_BLOCK, slot).await;

                        metrics
                            .increment_counter("block_crawler_blocks_received", 1)
                            .await
                            .unwrap_or_else(|value| {
                                log::error!("Error recording metric: {value}")
                            });
                        let block_start_time = Instant::now();
                        let block_hash = Hash::from_str(&block.blockhash).ok();
                        if let Some(transactions) = block.transactions {
                            let mut warned_backpressure_slots = HashSet::new();
                            update_stage_slot(&metrics, EXTRACT_TRANSACTIONS_FROM_BLOCK, slot)
                                .await;
                            for (tx_index, encoded_transaction_with_status_meta) in transactions.into_iter().enumerate() {
                                let start_time = std::time::Instant::now();
                                update_stage_slot(&metrics, DECODE_TRANSACTION, slot).await;

                                let meta_original = if let Some(meta) = encoded_transaction_with_status_meta.clone().meta {
                                    meta
                                } else {
                                    update_stage_slot(
                                        &metrics,
                                        SKIP_TRANSACTION_DECODE_OR_META_ERROR,
                                        slot,
                                    )
                                    .await;
                                    continue;
                                };

                                if meta_original.status.is_err() {
                                    update_stage_slot(
                                        &metrics,
                                        SKIP_TRANSACTION_FAILED_STATUS,
                                        slot,
                                    )
                                    .await;
                                    continue;
                                }

                                let Some(decoded_transaction) = encoded_transaction_with_status_meta.transaction.decode() else {
                                    log::error!("Failed to decode transaction: {encoded_transaction_with_status_meta:?}");
                                    update_stage_slot(
                                        &metrics,
                                        SKIP_TRANSACTION_DECODE_OR_META_ERROR,
                                        slot,
                                    )
                                    .await;
                                    continue;
                                };

                                update_stage_slot(&metrics, TRANSFORM_TRANSACTION_META, slot).await;
                                let Ok(meta_needed) = transaction_metadata_from_original_meta(meta_original) else {
                                    log::error!("Error getting metadata from transaction original meta.");
                                    update_stage_slot(
                                        &metrics,
                                        SKIP_TRANSACTION_DECODE_OR_META_ERROR,
                                        slot,
                                    )
                                    .await;
                                    continue;
                                };

                                update_stage_slot(&metrics, BUILD_TRANSACTION_UPDATE, slot).await;
                                let update = Update::Transaction(Box::new(TransactionUpdate {
                                    signature: *decoded_transaction.get_signature(),
                                    transaction: decoded_transaction.clone(),
                                    meta: meta_needed,
                                    is_vote: false,
                                    slot,
                                    index: Some(tx_index as u64),
                                    block_time: block.block_time,
                                    block_hash,
                                }));

                                metrics
                                    .record_histogram(
                                        "block_crawler_transaction_process_time_nanoseconds",
                                        start_time.elapsed().as_nanos() as f64
                                    )
                                    .await
                                    .unwrap_or_else(|value| log::error!("Error recording metric: {value}"));

                                metrics.increment_counter("block_crawler_transactions_processed", 1)
                                    .await
                                    .unwrap_or_else(|value| log::error!("Error recording metric: {value}"));

                                update_stage_slot(&metrics, ENQUEUE_PIPELINE_UPDATE, slot).await;
                                match sender.try_send((update, id_for_loop.clone())) {
                                    Ok(()) => {}
                                    Err(TrySendError::Full((update, datasource_id))) => {
                                        update_stage_slot(
                                            &metrics,
                                            DROP_OR_DELAY_ON_CHANNEL_BACKPRESSURE,
                                            slot,
                                        )
                                        .await;
                                        if warned_backpressure_slots.insert(slot) {
                                            log::warn!(
                                                "Transaction update channel full for slot {slot}; waiting for capacity"
                                            );
                                        }
                                        if let Err(e) = sender.send((update, datasource_id)).await {
                                            update_stage_slot(
                                                &metrics,
                                                ENQUEUE_PIPELINE_UPDATE_EXIT,
                                                slot,
                                            )
                                            .await;
                                            log::error!(
                                                "Error sending transaction update after backpressure wait: {e:?}"
                                            );
                                            break;
                                        }
                                    }
                                    Err(TrySendError::Closed((_update, _id))) => {
                                        update_stage_slot(
                                            &metrics,
                                            ENQUEUE_PIPELINE_UPDATE_EXIT,
                                            slot,
                                        )
                                        .await;
                                        log::error!("Error sending transaction update: \"Closed(..)\"");
                                        break;
                                    }
                                };
                            }
                            update_stage_slot(&metrics, EXTRACT_TRANSACTIONS_FROM_BLOCK_EXIT, slot)
                                .await;
                        }
                        metrics
                            .record_histogram(
                                "block_crawler_block_process_time_nanoseconds",
                                block_start_time.elapsed().as_nanos() as f64
                            ).await
                            .unwrap_or_else(|value| log::error!("Error recording metric: {value}"));

                        metrics
                            .increment_counter("block_crawler_blocks_processed", 1)
                            .await
                            .unwrap_or_else(|value| log::error!("Error recording metric: {value}"));
                    }
                    None => {
                        break;
                    }
                }
            }}
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_block_fetcher_with_end_slot() {
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            "https://api.mainnet-beta.solana.com/".to_string(),
            CommitmentConfig::confirmed(),
        ));
        let block_interval = Duration::from_millis(100);
        let cancellation_token = CancellationToken::new();
        let (block_sender, mut block_receiver) = mpsc::channel(1);

        let block_config = RpcBlockConfig {
            max_supported_transaction_version: Some(0),
            ..Default::default()
        };

        // Start block_fetcher
        let block_fetcher = block_fetcher(
            rpc_client,
            328837890,
            Some(328837901),
            block_interval,
            block_config,
            block_sender,
            1,
            cancellation_token.clone(),
            Arc::new(MetricsCollection::new(vec![])),
        );

        // Create a task to receive blocks
        let receiver_task = tokio::spawn(async move {
            let mut received_blocks = Vec::new();

            while let Some((slot, block)) = block_receiver.recv().await {
                received_blocks.push((slot, block));

                if received_blocks.len() == 2 {
                    break;
                }
            }
            received_blocks
        });

        tokio::spawn(async move {
            block_fetcher.await.expect("Block fetcher should not panic");
        });

        // Wait for both block_fetcher and receiver task to complete
        let exit_reason = tokio::select! {
            result = receiver_task => {
                let received_blocks = result.expect("Receiver task should not panic");
                println!("Received {} blocks", received_blocks.len());

                for (slot, block) in received_blocks {
                    println!("Block at slot {}: {} transactions",
                        slot,
                        block.transactions.map(|t| t.len()).unwrap_or(0)
                    );
                }
                "receiver_completed"
            }
            _ = cancellation_token.cancelled() => {
                println!("Cancellation token triggered");
                "cancellation_token"
            }
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                println!("Timeout");
                "timeout"
            }
        };

        assert_eq!(
            exit_reason, "receiver_completed",
            "Test should exit because block fetcher completed"
        );
    }

    #[tokio::test]
    async fn test_block_fetcher_without_end_slot() {
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            "https://api.mainnet-beta.solana.com/".to_string(),
            CommitmentConfig::confirmed(),
        ));
        let latest_slot = rpc_client
            .get_slot()
            .await
            .expect("Failed to get last slot");

        let block_interval = Duration::from_millis(100);
        let cancellation_token = CancellationToken::new();
        let (block_sender, mut block_receiver) = mpsc::channel(1);

        let block_config = RpcBlockConfig {
            max_supported_transaction_version: Some(0),
            ..Default::default()
        };

        // Start block_fetcher
        let block_fetcher = block_fetcher(
            rpc_client,
            latest_slot,
            None,
            block_interval,
            block_config,
            block_sender,
            2,
            cancellation_token.clone(),
            Arc::new(MetricsCollection::new(vec![])),
        );

        // Create a task to receive blocks
        let receiver_task = tokio::spawn(async move {
            let mut received_blocks = Vec::new();

            while let Some((slot, block)) = block_receiver.recv().await {
                println!("Received block at slot {slot}");
                received_blocks.push((slot, block));

                if received_blocks.len() == 2 {
                    break;
                }
            }
            received_blocks
        });

        tokio::spawn(async move {
            block_fetcher.await.expect("Block fetcher should not panic");
        });

        // Wait for both block_fetcher and receiver task to complete
        let exit_reason = tokio::select! {
            result = receiver_task => {
                let received_blocks = result.expect("Receiver task should not panic");
                println!("Received {} blocks", received_blocks.len());

                for (slot, block) in received_blocks {
                    println!("Block at slot {}: {} transactions",
                        slot,
                        block.transactions.map(|t| t.len()).unwrap_or(0)
                    );
                }
                "receiver_completed"
            }
            _ = cancellation_token.cancelled() => {
                println!("Cancellation token triggered");
                "cancellation_token"
            }
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                println!("Timeout");
                "timeout"
            }
        };

        assert_eq!(
            exit_reason, "receiver_completed",
            "Test should exit because block fetcher completed"
        );
    }
}
