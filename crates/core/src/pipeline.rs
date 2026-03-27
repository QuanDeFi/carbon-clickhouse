//! Defines the `Pipeline` struct and related components for processing
//! blockchain data updates.
//!
//! The `Pipeline` module is central to the `carbon-core` framework, offering a
//! flexible and extensible data processing architecture that supports various
//! blockchain data types, including account updates, transaction details, and
//! account deletions. The pipeline integrates multiple data sources and
//! processing pipes to handle and transform incoming data, while recording
//! performance metrics for monitoring and analysis.
//!
//! # Overview
//!
//! This module provides the `Pipeline` struct, which orchestrates data flow
//! from multiple sources, processes it through designated pipes, and captures
//! metrics at each stage. The pipeline is highly customizable and can be
//! configured with various components to suit specific data handling
//! requirements.
//!
//! ## Key Components
//!
//! - **Datasources**: Provide raw data updates, which may include account or
//!   transaction details.
//! - **Account, Instruction, and Transaction Pipes**: Modular units that decode
//!   and process specific types of data. Account pipes handle account updates,
//!   instruction pipes process instructions within transactions, and
//!   transaction pipes manage complete transaction records.
//! - **Metrics**: Collects data on pipeline performance, such as processing
//!   times and error rates, providing insights into operational efficiency.
//!
//! # Fields and Configuration
//!
//! - **datasources**: A list of `Datasource` objects that act as the sources
//!   for account and transaction data.
//! - **account_pipes**: A collection of pipes for processing account updates.
//! - **account_deletion_pipes**: Pipes responsible for handling account
//!   deletion events.
//! - **instruction_pipes**: Used to process instructions within transactions.
//! - **transaction_pipes**: For handling full transactions.
//! - **metrics**: A vector of `Metrics` implementations that gather and report
//!   on performance data.
//! - **metrics_flush_interval**: Specifies how frequently metrics are flushed.
//!   Defaults to 5 seconds if unset.
//!
//! ## Notes
//!
//! - Each pipe and data source must implement the appropriate traits
//!   (`Datasource`, `AccountPipes`, `Metrics`, etc.).
//! - The `Pipeline` is designed for concurrent operation, with `Arc` and `Box`
//!   wrappers ensuring safe, shared access.
//! - Proper metric collection and flushing are essential for monitoring
//!   pipeline performance, especially in production environments.

use crate::block_details::{BlockDetailsPipe, BlockDetailsPipes};
use crate::datasource::{BlockDetails, DatasourceId};
use crate::filter::Filter;
use {
    crate::{
        account::{
            AccountDecoder, AccountMetadata, AccountPipe, AccountPipes, AccountProcessorInputType,
        },
        account_deletion::{AccountDeletionPipe, AccountDeletionPipes},
        collection::InstructionDecoderCollection,
        datasource::{AccountDeletion, Datasource, Update},
        error::CarbonResult,
        instruction::{
            InstructionDecoder, InstructionPipe, InstructionPipes, InstructionProcessorInputType,
            InstructionsWithMetadata, NestedInstructions,
        },
        metrics::{Metrics, MetricsCollection},
        processor::Processor,
        schema::TransactionSchema,
        transaction::{TransactionPipe, TransactionPipes, TransactionProcessorInputType},
        transformers,
    },
    core::time,
    serde::de::DeserializeOwned,
    std::{
        convert::TryInto,
        env,
        sync::{Arc, Mutex, OnceLock},
        time::{Duration, Instant},
    },
    tokio_util::sync::CancellationToken,
};

/// Defines the shutdown behavior for the pipeline.
///
/// `ShutdownStrategy` determines how the pipeline will behave when it receives
/// a shutdown signal. It supports two modes:
///
/// - `Immediate`: Stops the entire pipeline, including all tasks, instantly.
/// - `ProcessPending`: Terminates the data sources, then completes processing
///   of any updates currently pending in the pipeline. This is the default
///   behavior.
///
/// # Variants
///
/// - `Immediate`: Immediately stops all pipeline activity without processing
///   any remaining updates.
/// - `ProcessPending`: Gracefully terminates the data sources and allows the
///   pipeline to finish processing updates that are still in progress or
///   queued.
///
/// # Notes
///
/// - `ProcessPending` is the default variant, enabling the pipeline to ensure
///   that no updates are lost during shutdown.
#[derive(Default, PartialEq, Debug)]
pub enum ShutdownStrategy {
    /// Stop the whole pipeline immediately.
    Immediate,
    /// Terminate the datasource(s) and finish processing all pending updates.
    #[default]
    ProcessPending,
}

/// The default size of the channel buffer for the pipeline.
///
/// This constant defines the default number of updates that can be queued in
/// the pipeline's channel buffer. It is used as a fallback value if the
/// `channel_buffer_size` is not explicitly set during pipeline construction.
///
/// The default size is 10,000 updates, which provides a reasonable balance
pub const DEFAULT_CHANNEL_BUFFER_SIZE: usize = 1_000;
const DEQUEUE_PIPELINE_UPDATE_STAGE: &str = "stage.dequeue_pipeline_update.slot";
const BUILD_CORE_TRANSACTION_METADATA_STAGE: &str = "stage.build_core_transaction_metadata.slot";
const EXTRACT_INSTRUCTIONS_WITH_METADATA_STAGE: &str =
    "stage.extract_instructions_with_metadata.slot";
const NEST_INSTRUCTIONS_STAGE: &str = "stage.nest_instructions.slot";
const APPLY_INSTRUCTION_FILTERS_STAGE: &str = "stage.apply_instruction_filters.slot";
const APPLY_INSTRUCTION_FILTERS_STAGE_EXIT: &str = "stage.apply_instruction_filters.exit.slot";
const PIPELINE_UPDATE_PROCESS_TIMEOUT_ENV: &str = "CARBON_PIPELINE_UPDATE_TIMEOUT_MS";
const INSTRUCTION_PIPE_TIMEOUT_ENV: &str = "CARBON_INSTRUCTION_PIPE_TIMEOUT_MS";

fn debug_stage_metrics_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var("CARBON_DEBUG_STAGE_METRICS")
            .map(|value| {
                matches!(
                    value.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    })
}

fn pipeline_update_process_timeout() -> Option<Duration> {
    static TIMEOUT: OnceLock<Option<Duration>> = OnceLock::new();
    *TIMEOUT.get_or_init(|| {
        env::var(PIPELINE_UPDATE_PROCESS_TIMEOUT_ENV)
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .map(Duration::from_millis)
    })
}

fn instruction_pipe_timeout() -> Option<Duration> {
    static TIMEOUT: OnceLock<Option<Duration>> = OnceLock::new();
    *TIMEOUT.get_or_init(|| {
        env::var(INSTRUCTION_PIPE_TIMEOUT_ENV)
            .ok()
            .and_then(|value| value.trim().parse::<u64>().ok())
            .filter(|value| *value > 0)
            .map(Duration::from_millis)
    })
}

async fn debug_inc(metrics: &MetricsCollection, name: &str, value: u64) {
    metrics
        .increment_counter(name, value)
        .await
        .unwrap_or_else(|error| log::error!("error recording debug counter {name}: {error:?}"));
}

async fn debug_hist(metrics: &MetricsCollection, name: &str, value: f64) {
    metrics
        .record_histogram(name, value)
        .await
        .unwrap_or_else(|error| log::error!("error recording debug histogram {name}: {error:?}"));
}

async fn debug_gauge(metrics: &MetricsCollection, name: &str, value: f64) {
    metrics
        .update_gauge(name, value)
        .await
        .unwrap_or_else(|error| log::error!("error recording debug gauge {name}: {error:?}"));
}

fn update_kind(update: &Update) -> &'static str {
    match update {
        Update::Account(_) => "account",
        Update::Transaction(_) => "transaction",
        Update::AccountDeletion(_) => "account_deletion",
        Update::BlockDetails(_) => "block_details",
    }
}

fn update_slot(update: &Update) -> Option<u64> {
    match update {
        Update::Account(account_update) => Some(account_update.slot),
        Update::Transaction(transaction_update) => Some(transaction_update.slot),
        Update::AccountDeletion(account_deletion) => Some(account_deletion.slot),
        Update::BlockDetails(block_details) => Some(block_details.slot),
    }
}

fn describe_update(update: &Update) -> String {
    match update {
        Update::Account(account_update) => {
            format!(
                "kind=account slot={} pubkey={}",
                account_update.slot, account_update.pubkey
            )
        }
        Update::Transaction(transaction_update) => {
            format!(
                "kind=transaction slot={} signature={}",
                transaction_update.slot, transaction_update.signature
            )
        }
        Update::AccountDeletion(account_deletion) => {
            format!(
                "kind=account_deletion slot={} pubkey={}",
                account_deletion.slot, account_deletion.pubkey
            )
        }
        Update::BlockDetails(block_details) => {
            format!("kind=block_details slot={}", block_details.slot)
        }
    }
}

fn is_instruction_pipe_timeout(error: &crate::error::Error) -> bool {
    matches!(
        error,
        crate::error::Error::Custom(message)
            if message.starts_with("instruction pipe timed out after ")
    )
}

#[derive(Debug, Clone)]
struct TransactionProgress {
    stage: &'static str,
    top_level_instruction_count: usize,
    instruction_entry_count: usize,
    nested_instruction_count: usize,
    current_instruction_index: Option<u32>,
    current_stack_height: Option<u32>,
    instruction_filter_checks: u64,
    instruction_pipe_runs: u64,
}

impl TransactionProgress {
    fn new() -> Self {
        Self {
            stage: "received",
            top_level_instruction_count: 0,
            instruction_entry_count: 0,
            nested_instruction_count: 0,
            current_instruction_index: None,
            current_stack_height: None,
            instruction_filter_checks: 0,
            instruction_pipe_runs: 0,
        }
    }

    fn describe(&self) -> String {
        format!(
            "stage={} top_level={} instruction_entries={} nested_roots={} current_instruction_index={:?} current_stack_height={:?} filter_checks={} pipe_runs={}",
            self.stage,
            self.top_level_instruction_count,
            self.instruction_entry_count,
            self.nested_instruction_count,
            self.current_instruction_index,
            self.current_stack_height,
            self.instruction_filter_checks,
            self.instruction_pipe_runs,
        )
    }
}

fn transaction_progress_stage(
    progress: &Option<Arc<Mutex<TransactionProgress>>>,
    stage: &'static str,
) {
    if let Some(progress) = progress {
        if let Ok(mut progress) = progress.lock() {
            progress.stage = stage;
        }
    }
}

fn transaction_progress_counts(
    progress: &Option<Arc<Mutex<TransactionProgress>>>,
    top_level_instruction_count: usize,
    instruction_entry_count: usize,
    nested_instruction_count: usize,
) {
    if let Some(progress) = progress {
        if let Ok(mut progress) = progress.lock() {
            progress.top_level_instruction_count = top_level_instruction_count;
            progress.instruction_entry_count = instruction_entry_count;
            progress.nested_instruction_count = nested_instruction_count;
        }
    }
}

fn transaction_progress_instruction(
    progress: &Option<Arc<Mutex<TransactionProgress>>>,
    instruction_index: u32,
    stack_height: u32,
) {
    if let Some(progress) = progress {
        if let Ok(mut progress) = progress.lock() {
            progress.current_instruction_index = Some(instruction_index);
            progress.current_stack_height = Some(stack_height);
        }
    }
}

fn transaction_progress_inc_filter_checks(progress: &Option<Arc<Mutex<TransactionProgress>>>) {
    if let Some(progress) = progress {
        if let Ok(mut progress) = progress.lock() {
            progress.instruction_filter_checks =
                progress.instruction_filter_checks.saturating_add(1);
        }
    }
}

fn transaction_progress_inc_pipe_runs(progress: &Option<Arc<Mutex<TransactionProgress>>>) {
    if let Some(progress) = progress {
        if let Ok(mut progress) = progress.lock() {
            progress.instruction_pipe_runs = progress.instruction_pipe_runs.saturating_add(1);
        }
    }
}

fn transaction_progress_context(
    progress: &Option<Arc<Mutex<TransactionProgress>>>,
) -> Option<String> {
    progress
        .as_ref()
        .and_then(|progress| progress.lock().ok().map(|progress| progress.describe()))
}

/// Represents the primary data processing pipeline in the `carbon-core`
/// framework.
///
/// The `Pipeline` struct is responsible for orchestrating the flow of data from
/// various sources, processing it through multiple pipes (for accounts,
/// transactions, instructions, and account deletions), and recording metrics at
/// each stage. This flexible design allows for customized data processing,
/// handling a variety of update types with minimal boilerplate code.
///
/// ## Overview
///
/// A `Pipeline` instance includes collections of data sources and processing
/// pipes, enabling users to configure the pipeline to handle diverse types of
/// blockchain-related data. Each pipe is responsible for decoding, processing,
/// and routing specific data types, while the metrics system records relevant
/// statistics.
///
/// ### Key Concepts
///
/// - **Datasources**: These provide the raw data, such as account updates,
///   transaction details, and account deletions.
/// - **Pipes**: Modular units that handle specific data types:
///   - `AccountPipes` for account updates.
///   - `AccountDeletionPipes` for account deletions.
///   - `InstructionPipes` for instruction data within transactions.
///   - `TransactionPipes` for entire transaction payloads.
/// - **Metrics**: Collect performance data, enabling real-time insights and
///   efficient monitoring.
///
/// ## Fields
///
/// - `datasources`: A vector of data sources (`Datasource` implementations)
///   that provide the data for processing. Each data source is paired with a
///   unique `DatasourceId` for identification and filtering purposes. Each data
///   source must be wrapped in an `Arc` for safe, concurrent access.
/// - `account_pipes`: A vector of `AccountPipes`, each responsible for handling
///   account updates.
/// - `account_deletion_pipes`: A vector of `AccountDeletionPipes` to handle
///   deletion events.
/// - `block_details_pipes`: A vector of `BlockDetailsPipes` to handle
///   block details.
/// - `instruction_pipes`: A vector of `InstructionPipes` for processing
///   instructions within transactions. These pipes work with nested
///   instructions and are generically defined to support varied instruction
///   types.
/// - `transaction_pipes`: A vector of `TransactionPipes` responsible for
///   processing complete transaction payloads.
/// - `metrics`: A vector of `Metrics` implementations to record and track
///   performance data. Each metrics instance is managed within an `Arc` to
///   ensure thread safety.
/// - `metrics_flush_interval`: An optional interval, in seconds, defining how
///   frequently metrics should be flushed. If `None`, the default interval is
///   used.
/// - `channel_buffer_size`: The size of the channel buffer for the pipeline. If
///   not set, a default size of 10_000 will be used.
///
/// ## Example
///
/// ```ignore
/// use std::sync::Arc;
///
/// carbon_core::pipeline::Pipeline::builder()
/// .datasource(transaction_crawler)
/// .metrics(Arc::new(LogMetrics::new()))
/// .metrics(Arc::new(PrometheusMetrics::new()))
/// .instruction(
///    TestProgramDecoder,
///    TestProgramProcessor
/// )
/// .account(
///     TestProgramDecoder,
///     TestProgramAccountProcessor
/// )
/// .transaction(TEST_SCHEMA.clone(), TestProgramTransactionProcessor)
/// .account_deletions(TestProgramAccountDeletionProcessor)
/// .channel_buffer_size(1000)
/// .build()?
/// .run()
/// .await?;
/// ```
///
/// ## Notes
///
/// - Ensure that each data source and pipe implements the required traits, such
///   as `Datasource`, `AccountPipes`, and `Metrics`, as appropriate.
/// - The pipeline is designed for concurrent operation, utilizing `Arc` and
///   `Box` types to handle shared ownership and trait object storage.
/// - The `metrics_flush_interval` controls how frequently the pipeline's
///   metrics are flushed. If `None`, a default interval (usually 5 seconds) is
///   used.
pub struct Pipeline {
    pub datasources: Vec<(DatasourceId, Arc<dyn Datasource + Send + Sync>)>,
    pub account_pipes: Vec<Box<dyn AccountPipes>>,
    pub account_deletion_pipes: Vec<Box<dyn AccountDeletionPipes>>,
    pub block_details_pipes: Vec<Box<dyn BlockDetailsPipes>>,
    pub instruction_pipes: Vec<Box<dyn for<'a> InstructionPipes<'a>>>,
    pub transaction_pipes: Vec<Box<dyn for<'a> TransactionPipes<'a>>>,
    pub metrics: Arc<MetricsCollection>,
    pub metrics_flush_interval: Option<u64>,
    pub datasource_cancellation_token: Option<CancellationToken>,
    pub shutdown_strategy: ShutdownStrategy,
    pub channel_buffer_size: usize,
}

impl Pipeline {
    /// Creates a new `PipelineBuilder` instance for constructing a `Pipeline`.
    ///
    /// The `builder` method returns a `PipelineBuilder` that allows you to
    /// configure and customize the pipeline components before building the
    /// final `Pipeline` object. This approach provides a flexible and
    /// type-safe way to assemble a pipeline by specifying data sources,
    /// processing pipes, and metrics.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::sync::Arc;
    ///
    /// carbon_core::pipeline::Pipeline::builder()
    /// .datasource(transaction_crawler)
    /// .metrics(Arc::new(LogMetrics::new()))
    /// .metrics(Arc::new(PrometheusMetrics::new()))
    /// .instruction(
    ///    TestProgramDecoder,
    ///    TestProgramProcessor
    /// );
    /// // ...
    /// ```
    ///
    /// # Returns
    ///
    /// Returns a `PipelineBuilder` instance with empty collections for data
    /// sources, pipes, and metrics. You can then configure each component
    /// using the builder pattern.
    pub fn builder() -> PipelineBuilder {
        log::trace!("Pipeline::builder()");
        PipelineBuilder {
            datasources: Vec::new(),
            account_pipes: Vec::new(),
            account_deletion_pipes: Vec::new(),
            block_details_pipes: Vec::new(),
            instruction_pipes: Vec::new(),
            transaction_pipes: Vec::new(),
            metrics: MetricsCollection::default(),
            metrics_flush_interval: None,
            datasource_cancellation_token: None,
            shutdown_strategy: ShutdownStrategy::default(),
            channel_buffer_size: DEFAULT_CHANNEL_BUFFER_SIZE,
        }
    }

    /// Runs the `Pipeline`, processing updates from data sources and handling
    /// metrics.
    ///
    /// The `run` method initializes the pipeline's metrics system and starts
    /// listening for updates from the configured data sources. It checks
    /// the types of updates provided by each data source to ensure that the
    /// required data types are available for processing. The method then
    /// enters a loop where it processes each update received from the data
    /// sources in turn, logging and updating metrics based on the success
    /// or failure of each operation.
    ///
    /// # How it Works
    ///
    /// - Initializes metrics and sets up an interval for periodic metric
    ///   flushing.
    /// - Spawns tasks for each data source to continuously consume updates.
    /// - Processes updates according to their type (e.g., Account, Transaction,
    ///   or AccountDeletion).
    /// - Records performance metrics such as update processing times, and
    ///   tracks success and failure counts.
    ///
    /// # Errors
    ///
    /// The method returns an `Err` variant if:
    /// - Required update types (e.g., `AccountUpdate`, `AccountDeletion`,
    ///   `Transaction`) are not provided by any data source, causing a mismatch
    ///   in expected data processing capabilities.
    /// - A data source encounters an error while consuming updates.
    /// - An error occurs during metrics flushing or processing of updates.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::pipeline::Pipeline;
    ///
    /// let mut pipeline = Pipeline::builder()
    ///     .datasource(MyDatasource::new())
    ///     .metrics(MyMetrics::new())
    ///     .build()
    ///     .expect("Failed to build pipeline");
    ///
    /// // Running the pipeline asynchronously
    /// tokio::spawn(async move {
    ///     if let Err(e) = pipeline.run().await {
    ///         eprintln!("Pipeline run error: {:?}", e);
    ///     }
    /// });
    /// ```
    ///
    /// # Notes
    ///
    /// - This method is asynchronous and should be awaited within a Tokio
    ///   runtime environment.
    /// - The pipeline monitors metrics and flushes them based on the configured
    ///   `metrics_flush_interval`.
    /// - The `run` method operates in an infinite loop, handling updates until
    ///   a termination condition occurs.
    pub async fn run(&mut self) -> CarbonResult<()> {
        log::info!("starting pipeline. num_datasources: {}, num_metrics: {}, num_account_pipes: {}, num_account_deletion_pipes: {}, num_instruction_pipes: {}, num_transaction_pipes: {}",
            self.datasources.len(),
            self.metrics.metrics.len(),
            self.account_pipes.len(),
            self.account_deletion_pipes.len(),
            self.instruction_pipes.len(),
            self.transaction_pipes.len(),
        );

        log::trace!("run(self)");
        let debug_stage_metrics = debug_stage_metrics_enabled();
        if debug_stage_metrics {
            log::info!("CARBON_DEBUG_STAGE_METRICS enabled");
        }

        self.metrics.initialize_metrics().await?;
        let (update_sender, mut update_receiver) =
            tokio::sync::mpsc::channel::<(Update, DatasourceId)>(self.channel_buffer_size);

        let datasource_cancellation_token = self
            .datasource_cancellation_token
            .clone()
            .unwrap_or_default();

        for datasource in &self.datasources {
            let datasource_cancellation_token_clone = datasource_cancellation_token.clone();
            let sender_clone = update_sender.clone();
            let datasource_clone = Arc::clone(&datasource.1);
            let datasource_id = datasource.0.clone();
            let metrics_collection = self.metrics.clone();

            tokio::spawn(async move {
                if let Err(e) = datasource_clone
                    .consume(
                        datasource_id,
                        sender_clone,
                        datasource_cancellation_token_clone,
                        metrics_collection,
                    )
                    .await
                {
                    log::error!("error consuming datasource: {e:?}");
                }
            });
        }

        drop(update_sender);

        let metrics_flush_interval_secs = self.metrics_flush_interval.unwrap_or(5);
        let metrics_flush_token = CancellationToken::new();
        let metrics_flush_token_clone = metrics_flush_token.clone();
        let metrics_for_flush = self.metrics.clone();
        let metrics_flush_task = tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(time::Duration::from_secs(metrics_flush_interval_secs));
            loop {
                tokio::select! {
                    _ = metrics_flush_token_clone.cancelled() => {
                        break;
                    }
                    _ = interval.tick() => {
                        if let Err(error) = metrics_for_flush.flush_metrics().await {
                            log::error!("error flushing metrics: {error:?}");
                        }
                    }
                }
            }
        });

        loop {
            tokio::select! {
                _ = datasource_cancellation_token.cancelled() => {
                    log::trace!("datasource cancellation token cancelled, shutting down.");
                    metrics_flush_token.cancel();
                    let _ = metrics_flush_task.await;
                    self.metrics.flush_metrics().await?;
                    self.metrics.shutdown_metrics().await?;
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    log::trace!("received SIGINT, shutting down.");
                    datasource_cancellation_token.cancel();

                    if self.shutdown_strategy == ShutdownStrategy::Immediate {
                        log::info!("shutting down the pipeline immediately.");
                        metrics_flush_token.cancel();
                        let _ = metrics_flush_task.await;
                        self.metrics.flush_metrics().await?;
                        self.metrics.shutdown_metrics().await?;
                        break;
                    } else {
                        log::info!("shutting down the pipeline after processing pending updates.");
                    }
                }
                update = update_receiver.recv() => {
                    match update {
                        Some((update, datasource_id)) => {
                            let update_kind = update_kind(&update);
                            let transaction_progress = matches!(update, Update::Transaction(_))
                                .then(|| Arc::new(Mutex::new(TransactionProgress::new())));
                            if debug_stage_metrics {
                                debug_inc(
                                    &self.metrics,
                                    &format!("core.pipeline.debug.stage.{update_kind}.received"),
                                    1,
                                )
                                .await;
                                debug_gauge(
                                    &self.metrics,
                                    "core.pipeline.debug.gauge.updates_queued_before_process",
                                    update_receiver.len() as f64,
                                )
                                .await;
                            }
                            if let Some(slot) = update_slot(&update) {
                                self.metrics
                                    .update_gauge(DEQUEUE_PIPELINE_UPDATE_STAGE, slot as f64)
                                    .await?;
                            }
                            self
                                .metrics.increment_counter("updates_received", 1)
                                .await?;

                            let start = Instant::now();
                            let process_result = if let Some(timeout) = pipeline_update_process_timeout() {
                                match tokio::time::timeout(
                                    timeout,
                                    self.process(
                                        update.clone(),
                                        datasource_id.clone(),
                                        transaction_progress.clone(),
                                    ),
                                )
                                .await
                                {
                                    Ok(result) => result,
                                    Err(_) => Err(crate::error::Error::Custom(format!(
                                        "pipeline update processing timed out after {} ms (kind={update_kind}, slot={:?}{}{})",
                                        timeout.as_millis(),
                                        update_slot(&update),
                                        if transaction_progress.is_some() { ", " } else { "" },
                                        transaction_progress_context(&transaction_progress)
                                            .unwrap_or_default(),
                                    ))),
                                }
                            } else {
                                self.process(
                                    update.clone(),
                                    datasource_id.clone(),
                                    transaction_progress.clone(),
                                )
                                .await
                            };
                            let time_taken_nanoseconds = start.elapsed().as_nanos();
                            let time_taken_milliseconds = time_taken_nanoseconds / 1_000_000;
                            if debug_stage_metrics {
                                debug_hist(
                                    &self.metrics,
                                    &format!("core.pipeline.debug.latency.{update_kind}.process_nanoseconds"),
                                    time_taken_nanoseconds as f64,
                                )
                                .await;
                            }

                            self
                                .metrics
                                .record_histogram("updates_process_time_nanoseconds", time_taken_nanoseconds as f64)
                                .await?;

                            self
                                .metrics
                                .record_histogram("updates_process_time_milliseconds", time_taken_milliseconds as f64)
                                .await?;

                            match process_result {
                                Ok(_) => {
                                    if debug_stage_metrics {
                                        debug_inc(
                                            &self.metrics,
                                            &format!("core.pipeline.debug.stage.{update_kind}.processed_success"),
                                            1,
                                        )
                                        .await;
                                    }
                                    self
                                        .metrics.increment_counter("updates_successful", 1)
                                        .await?;

                                    log::trace!("processed update")
                                }
                                Err(error) => {
                                    log::error!(
                                        "error processing update {}: {error:?}",
                                        describe_update(&update)
                                    );
                                    if debug_stage_metrics {
                                        debug_inc(
                                            &self.metrics,
                                            &format!("core.pipeline.debug.stage.{update_kind}.processed_error"),
                                            1,
                                        )
                                        .await;
                                    }
                                    self.metrics.increment_counter("updates_failed", 1).await?;
                                }
                            };

                            self
                                .metrics.increment_counter("updates_processed", 1)
                                .await?;

                            self
                                .metrics.update_gauge("updates_queued", update_receiver.len() as f64)
                                .await?;
                            if debug_stage_metrics {
                                debug_gauge(
                                    &self.metrics,
                                    "core.pipeline.debug.gauge.updates_queued_after_process",
                                    update_receiver.len() as f64,
                                )
                                .await;
                            }
                        }
                        None => {
                            log::info!("update_receiver closed, shutting down.");
                            metrics_flush_token.cancel();
                            let _ = metrics_flush_task.await;
                            self.metrics.flush_metrics().await?;
                            self.metrics.shutdown_metrics().await?;
                            break;
                        }
                    }
                }
            }
        }

        log::info!("pipeline shutdown complete.");

        Ok(())
    }

    /// Processes a single update and routes it through the appropriate pipeline
    /// stages.
    ///
    /// The `process` method takes an `Update` and determines its type, then
    /// routes it through the corresponding pipes for handling account
    /// updates, transactions, or account deletions. It also records metrics
    /// for processed updates, providing insights into the processing
    /// workload and performance.
    ///
    /// ## Functionality
    ///
    /// - **Account Updates**: Passes account updates through the
    ///   `account_pipes`. Each pipe processes the account metadata and the
    ///   updated account state.
    /// - **Transaction Updates**: Extracts transaction metadata and
    ///   instructions, nests them if needed, and routes them through
    ///   `instruction_pipes` and `transaction_pipes`.
    /// - **Account Deletions**: Sends account deletion events through the
    ///   `account_deletion_pipes`.
    ///
    /// The method also updates metrics counters for each type of update,
    /// tracking how many updates have been processed in each category.
    ///
    /// # Parameters
    ///
    /// - `update`: An `Update` variant representing the type of data received.
    ///   This can be an `Account`, `Transaction`, or `AccountDeletion`, each
    ///   triggering different processing logic within the pipeline.
    /// - `datasource_id`: The ID of the datasource that produced this update.
    ///   This is used by filters to determine whether the update should be
    ///   processed by specific pipes.
    ///
    /// # Returns
    ///
    /// Returns a `CarbonResult<()>`, indicating `Ok(())` on successful
    /// processing or an error if processing fails at any stage.
    ///
    /// # Notes
    ///
    /// - This method is asynchronous and should be awaited within a Tokio
    ///   runtime.
    /// - Each type of update (account, transaction, account deletion) requires
    ///   its own set of pipes, so ensure that appropriate pipes are configured
    ///   based on the data types expected from the data sources.
    /// - Metrics are recorded after each successful processing stage to track
    ///   processing volumes and identify potential bottlenecks in real-time.
    /// - Filters are applied to each pipe before processing, allowing for
    ///   selective update processing based on datasource ID and other criteria.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the pipes fail during processing, or if an
    /// issue arises while incrementing counters or updating metrics. Handle
    /// errors gracefully to ensure continuous pipeline operation.
    async fn process(
        &mut self,
        update: Update,
        datasource_id: DatasourceId,
        transaction_progress: Option<Arc<Mutex<TransactionProgress>>>,
    ) -> CarbonResult<()> {
        log::trace!("process(self, update: {update:?}, datasource_id: {datasource_id:?})");
        let debug_stage_metrics = debug_stage_metrics_enabled();
        match update {
            Update::Account(account_update) => {
                if debug_stage_metrics {
                    debug_inc(&self.metrics, "core.pipeline.debug.stage.account.enter", 1).await;
                    debug_gauge(
                        &self.metrics,
                        "core.pipeline.debug.gauge.account.slot",
                        account_update.slot as f64,
                    )
                    .await;
                }
                let account_metadata = AccountMetadata {
                    slot: account_update.slot,
                    pubkey: account_update.pubkey,
                    transaction_signature: account_update.transaction_signature,
                };

                for pipe in self.account_pipes.iter_mut() {
                    if pipe.filters().iter().all(|filter| {
                        filter.filter_account(
                            &datasource_id,
                            &account_metadata,
                            &account_update.account,
                        )
                    }) {
                        if debug_stage_metrics {
                            debug_inc(
                                &self.metrics,
                                "core.pipeline.debug.stage.account.pipe_filter_passed",
                                1,
                            )
                            .await;
                        }
                        let stage_start = Instant::now();
                        let run_result = pipe
                            .run(
                                (account_metadata.clone(), account_update.account.clone()),
                                self.metrics.clone(),
                            )
                            .await;
                        if debug_stage_metrics {
                            debug_hist(
                                &self.metrics,
                                "core.pipeline.debug.latency.account.pipe_run_nanoseconds",
                                stage_start.elapsed().as_nanos() as f64,
                            )
                            .await;
                        }
                        run_result?;
                    } else if debug_stage_metrics {
                        debug_inc(
                            &self.metrics,
                            "core.pipeline.debug.stage.account.pipe_filter_dropped",
                            1,
                        )
                        .await;
                    }
                }

                self.metrics
                    .increment_counter("account_updates_processed", 1)
                    .await?;
            }
            Update::Transaction(transaction_update) => {
                transaction_progress_stage(&transaction_progress, "transaction_enter");
                if debug_stage_metrics {
                    debug_inc(
                        &self.metrics,
                        "core.pipeline.debug.stage.transaction.enter",
                        1,
                    )
                    .await;
                    debug_gauge(
                        &self.metrics,
                        "core.pipeline.debug.gauge.transaction.slot",
                        transaction_update.slot as f64,
                    )
                    .await;
                }
                transaction_progress_stage(&transaction_progress, "metadata_build");
                let metadata_build_start = Instant::now();
                let transaction_metadata: Arc<crate::transaction::TransactionMetadata> =
                    Arc::new((*transaction_update).clone().try_into()?);
                self.metrics
                    .update_gauge(
                        BUILD_CORE_TRANSACTION_METADATA_STAGE,
                        transaction_metadata.slot as f64,
                    )
                    .await?;
                if debug_stage_metrics {
                    debug_inc(
                        &self.metrics,
                        "core.pipeline.debug.stage.transaction.metadata_built",
                        1,
                    )
                    .await;
                    debug_hist(
                        &self.metrics,
                        "core.pipeline.debug.latency.transaction.metadata_build_nanoseconds",
                        metadata_build_start.elapsed().as_nanos() as f64,
                    )
                    .await;
                }

                let top_level_instruction_count = match &transaction_update.transaction.message {
                    solana_message::VersionedMessage::Legacy(message) => message.instructions.len(),
                    solana_message::VersionedMessage::V0(message) => message.instructions.len(),
                };
                transaction_progress_stage(&transaction_progress, "instruction_extract");
                let extract_start = Instant::now();
                let instructions_with_metadata: InstructionsWithMetadata =
                    transformers::extract_instructions_with_metadata(
                        &transaction_metadata,
                        &transaction_update,
                    )?;
                self.metrics
                    .update_gauge(
                        EXTRACT_INSTRUCTIONS_WITH_METADATA_STAGE,
                        transaction_metadata.slot as f64,
                    )
                    .await?;
                if debug_stage_metrics {
                    debug_inc(
                        &self.metrics,
                        "core.pipeline.debug.stage.transaction.instructions_extracted",
                        1,
                    )
                    .await;
                    debug_hist(
                        &self.metrics,
                        "core.pipeline.debug.latency.transaction.instructions_extract_nanoseconds",
                        extract_start.elapsed().as_nanos() as f64,
                    )
                    .await;
                }

                transaction_progress_stage(&transaction_progress, "instruction_nest");
                let nest_start = Instant::now();
                let instruction_entry_count = instructions_with_metadata.len();
                let nested_instructions: NestedInstructions = instructions_with_metadata.into();
                transaction_progress_counts(
                    &transaction_progress,
                    top_level_instruction_count,
                    instruction_entry_count,
                    nested_instructions.len(),
                );
                self.metrics
                    .update_gauge(NEST_INSTRUCTIONS_STAGE, transaction_metadata.slot as f64)
                    .await?;
                if debug_stage_metrics {
                    debug_inc(
                        &self.metrics,
                        "core.pipeline.debug.stage.transaction.instructions_nested",
                        1,
                    )
                    .await;
                    debug_hist(
                        &self.metrics,
                        "core.pipeline.debug.latency.transaction.instructions_nest_nanoseconds",
                        nest_start.elapsed().as_nanos() as f64,
                    )
                    .await;
                    debug_gauge(
                        &self.metrics,
                        "core.pipeline.debug.gauge.transaction.nested_instruction_count",
                        nested_instructions.len() as f64,
                    )
                    .await;
                }

                transaction_progress_stage(&transaction_progress, "instruction_pipe_loop");
                for pipe in self.instruction_pipes.iter_mut() {
                    for nested_instruction in nested_instructions.iter() {
                        transaction_progress_instruction(
                            &transaction_progress,
                            nested_instruction.metadata.index,
                            nested_instruction.metadata.stack_height,
                        );
                        self.metrics
                            .update_gauge(
                                APPLY_INSTRUCTION_FILTERS_STAGE,
                                nested_instruction.metadata.transaction_metadata.slot as f64,
                            )
                            .await?;
                        if debug_stage_metrics {
                            debug_inc(
                                &self.metrics,
                                "core.pipeline.debug.stage.instruction.filter_checks",
                                1,
                            )
                            .await;
                        }
                        transaction_progress_inc_filter_checks(&transaction_progress);
                        if pipe.filters().iter().all(|filter| {
                            filter.filter_instruction(&datasource_id, nested_instruction)
                        }) {
                            if debug_stage_metrics {
                                debug_inc(
                                    &self.metrics,
                                    "core.pipeline.debug.stage.instruction.filter_passed",
                                    1,
                                )
                                .await;
                            }
                            transaction_progress_stage(
                                &transaction_progress,
                                "instruction_pipe_run",
                            );
                            transaction_progress_inc_pipe_runs(&transaction_progress);
                            let instruction_pipe_start = Instant::now();
                            let run_result = if let Some(timeout) = instruction_pipe_timeout() {
                                match tokio::time::timeout(
                                    timeout,
                                    pipe.run(nested_instruction, self.metrics.clone()),
                                )
                                .await
                                {
                                    Ok(result) => result,
                                    Err(_) => Err(crate::error::Error::Custom(format!(
                                        "instruction pipe timed out after {} ms (slot={}, instruction_index={}, stack_height={})",
                                        timeout.as_millis(),
                                        nested_instruction.metadata.transaction_metadata.slot,
                                        nested_instruction.metadata.index,
                                        nested_instruction.metadata.stack_height,
                                    ))),
                                }
                            } else {
                                pipe.run(nested_instruction, self.metrics.clone()).await
                            };
                            if debug_stage_metrics {
                                debug_hist(
                                    &self.metrics,
                                    "core.pipeline.debug.latency.instruction.pipe_run_nanoseconds",
                                    instruction_pipe_start.elapsed().as_nanos() as f64,
                                )
                                .await;
                            }
                            match run_result {
                                Ok(()) => {
                                    if debug_stage_metrics {
                                        debug_inc(
                                            &self.metrics,
                                            "core.pipeline.debug.stage.instruction.pipe_run_success",
                                            1,
                                        )
                                        .await;
                                    }
                                }
                                Err(error) => {
                                    if debug_stage_metrics {
                                        debug_inc(
                                            &self.metrics,
                                            "core.pipeline.debug.stage.instruction.pipe_run_error",
                                            1,
                                        )
                                        .await;
                                    }
                                    if is_instruction_pipe_timeout(&error) {
                                        log::warn!(
                                            "skipping timed out instruction slot={} instruction_index={} stack_height={}: {error:?}",
                                            nested_instruction.metadata.transaction_metadata.slot,
                                            nested_instruction.metadata.index,
                                            nested_instruction.metadata.stack_height,
                                        );
                                        transaction_progress_stage(
                                            &transaction_progress,
                                            "instruction_pipe_timeout_skipped",
                                        );
                                        continue;
                                    }
                                    return Err(error);
                                }
                            }
                            transaction_progress_stage(&transaction_progress, "instruction_pipe_loop");
                        } else {
                            if debug_stage_metrics {
                                debug_inc(
                                    &self.metrics,
                                    "core.pipeline.debug.stage.instruction.filter_dropped",
                                    1,
                                )
                                .await;
                            }
                            transaction_progress_stage(
                                &transaction_progress,
                                "instruction_filter_dropped",
                            );
                            self.metrics
                                .update_gauge(
                                    APPLY_INSTRUCTION_FILTERS_STAGE_EXIT,
                                    nested_instruction.metadata.transaction_metadata.slot as f64,
                                )
                                .await?;
                        }
                    }
                }

                transaction_progress_stage(&transaction_progress, "transaction_pipe_loop");
                for pipe in self.transaction_pipes.iter_mut() {
                    if debug_stage_metrics {
                        debug_inc(
                            &self.metrics,
                            "core.pipeline.debug.stage.transaction.pipe_filter_checks",
                            1,
                        )
                        .await;
                    }
                    if pipe.filters().iter().all(|filter| {
                        filter.filter_transaction(
                            &datasource_id,
                            &transaction_metadata,
                            &nested_instructions,
                        )
                    }) {
                        if debug_stage_metrics {
                            debug_inc(
                                &self.metrics,
                                "core.pipeline.debug.stage.transaction.pipe_filter_passed",
                                1,
                            )
                            .await;
                        }
                        let transaction_pipe_start = Instant::now();
                        let run_result = pipe
                            .run(
                                transaction_metadata.clone(),
                                &nested_instructions,
                                self.metrics.clone(),
                            )
                            .await;
                        if debug_stage_metrics {
                            debug_hist(
                                &self.metrics,
                                "core.pipeline.debug.latency.transaction.pipe_run_nanoseconds",
                                transaction_pipe_start.elapsed().as_nanos() as f64,
                            )
                            .await;
                        }
                        match run_result {
                            Ok(()) => {
                                if debug_stage_metrics {
                                    debug_inc(
                                        &self.metrics,
                                        "core.pipeline.debug.stage.transaction.pipe_run_success",
                                        1,
                                    )
                                    .await;
                                }
                            }
                            Err(error) => {
                                if debug_stage_metrics {
                                    debug_inc(
                                        &self.metrics,
                                        "core.pipeline.debug.stage.transaction.pipe_run_error",
                                        1,
                                    )
                                    .await;
                                }
                                return Err(error);
                            }
                        }
                    } else if debug_stage_metrics {
                        debug_inc(
                            &self.metrics,
                            "core.pipeline.debug.stage.transaction.pipe_filter_dropped",
                            1,
                        )
                        .await;
                    }
                }

                transaction_progress_stage(&transaction_progress, "transaction_complete");
                self.metrics
                    .increment_counter("transaction_updates_processed", 1)
                    .await?;
            }
            Update::AccountDeletion(account_deletion) => {
                if debug_stage_metrics {
                    debug_inc(
                        &self.metrics,
                        "core.pipeline.debug.stage.account_deletion.enter",
                        1,
                    )
                    .await;
                }
                for pipe in self.account_deletion_pipes.iter_mut() {
                    if pipe.filters().iter().all(|filter| {
                        filter.filter_account_deletion(&datasource_id, &account_deletion)
                    }) {
                        if debug_stage_metrics {
                            debug_inc(
                                &self.metrics,
                                "core.pipeline.debug.stage.account_deletion.pipe_filter_passed",
                                1,
                            )
                            .await;
                        }
                        let stage_start = Instant::now();
                        let run_result = pipe
                            .run(account_deletion.clone(), self.metrics.clone())
                            .await;
                        if debug_stage_metrics {
                            debug_hist(
                                &self.metrics,
                                "core.pipeline.debug.latency.account_deletion.pipe_run_nanoseconds",
                                stage_start.elapsed().as_nanos() as f64,
                            )
                            .await;
                        }
                        run_result?;
                    } else if debug_stage_metrics {
                        debug_inc(
                            &self.metrics,
                            "core.pipeline.debug.stage.account_deletion.pipe_filter_dropped",
                            1,
                        )
                        .await;
                    }
                }

                self.metrics
                    .increment_counter("account_deletions_processed", 1)
                    .await?;
            }
            Update::BlockDetails(block_details) => {
                if debug_stage_metrics {
                    debug_inc(
                        &self.metrics,
                        "core.pipeline.debug.stage.block_details.enter",
                        1,
                    )
                    .await;
                    debug_gauge(
                        &self.metrics,
                        "core.pipeline.debug.gauge.block_details.slot",
                        block_details.slot as f64,
                    )
                    .await;
                }
                for pipe in self.block_details_pipes.iter_mut() {
                    if pipe
                        .filters()
                        .iter()
                        .all(|filter| filter.filter_block_details(&datasource_id, &block_details))
                    {
                        if debug_stage_metrics {
                            debug_inc(
                                &self.metrics,
                                "core.pipeline.debug.stage.block_details.pipe_filter_passed",
                                1,
                            )
                            .await;
                        }
                        let stage_start = Instant::now();
                        let run_result =
                            pipe.run(block_details.clone(), self.metrics.clone()).await;
                        if debug_stage_metrics {
                            debug_hist(
                                &self.metrics,
                                "core.pipeline.debug.latency.block_details.pipe_run_nanoseconds",
                                stage_start.elapsed().as_nanos() as f64,
                            )
                            .await;
                        }
                        run_result?;
                    } else if debug_stage_metrics {
                        debug_inc(
                            &self.metrics,
                            "core.pipeline.debug.stage.block_details.pipe_filter_dropped",
                            1,
                        )
                        .await;
                    }
                }

                self.metrics
                    .increment_counter("block_details_processed", 1)
                    .await?;
            }
        };

        Ok(())
    }
}

/// A builder for constructing a `Pipeline` instance with customized data
/// sources, processing pipes, and metrics.
///
/// The `PipelineBuilder` struct offers a flexible way to assemble a `Pipeline`
/// by allowing configuration of its components, such as data sources, account
/// and transaction pipes, deletion handling, and metrics. Using the builder
/// pattern, you can add the desired elements incrementally and then finalize
/// with a call to `build`.
///
/// ## Overview
///
/// The `PipelineBuilder` supports the following components:
/// - **Datasources**: Sources of data updates, such as account information and
///   transactions.
/// - **Account Pipes**: For processing account updates from data sources.
/// - **Account Deletion Pipes**: For handling account deletion updates.
/// - **Instruction Pipes**: For handling instructions associated with
///   transactions.
/// - **Transaction Pipes**: For handling full transaction data.
/// - **Metrics**: Collects and reports performance data, such as update
///   processing times.
/// - **Metrics Flush Interval**: Optional interval defining how often to flush
///   metrics data.
///
/// Each component can be added through method chaining, enhancing code
/// readability and maintainability.
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
///
/// carbon_core::pipeline::Pipeline::builder()
/// .datasource(transaction_crawler)
/// .metrics(Arc::new(LogMetrics::new()))
/// .metrics(Arc::new(PrometheusMetrics::new()))
/// .instruction(
///    TestProgramDecoder,
///    TestProgramProcessor
/// );
/// // ...
/// ```
///
/// # Fields
///
/// - `datasources`: A collection of `Datasource` objects wrapped in `Arc` for
///   shared ownership across threads. Each `Datasource` provides updates to the
///   pipeline.
/// - `account_pipes`: A collection of `AccountPipes` to handle account updates.
/// - `account_deletion_pipes`: A collection of `AccountDeletionPipes` for
///   processing account deletions.
/// - `instruction_pipes`: A collection of `InstructionPipes` to process
///   instructions in transactions.
/// - `transaction_pipes`: A collection of `TransactionPipes` to process full
///   transaction data.
/// - `metrics`: A vector of `Metrics` implementations for tracking pipeline
///   performance.
/// - `metrics_flush_interval`: An optional interval (in seconds) for flushing
///   metrics data. If not set, a default flush interval will be used.
/// - `datasource_cancellation_token`: An optional `CancellationToken` for
///   canceling datasource. If not set, a default `CancellationToken` will be
///   used.
/// - `channel_buffer_size`: The size of the channel buffer for the pipeline. If
///   not set, a default size of 10_000 will be used.
///
/// # Returns
///
/// After configuring the builder, call `build` to create a `Pipeline` instance.
/// The builder will return a `CarbonResult<Pipeline>`, which will either
/// contain the configured pipeline or an error if configuration failed.
///
/// # Notes
///
/// - The builder pattern allows for method chaining, making it easy to
///   incrementally add components to the `Pipeline`.
/// - Ensure that each component matches the data and update types expected by
///   your application.
#[derive(Default)]
pub struct PipelineBuilder {
    pub datasources: Vec<(DatasourceId, Arc<dyn Datasource + Send + Sync>)>,
    pub account_pipes: Vec<Box<dyn AccountPipes>>,
    pub account_deletion_pipes: Vec<Box<dyn AccountDeletionPipes>>,
    pub block_details_pipes: Vec<Box<dyn BlockDetailsPipes>>,
    pub instruction_pipes: Vec<Box<dyn for<'a> InstructionPipes<'a>>>,
    pub transaction_pipes: Vec<Box<dyn for<'a> TransactionPipes<'a>>>,
    pub metrics: MetricsCollection,
    pub metrics_flush_interval: Option<u64>,
    pub datasource_cancellation_token: Option<CancellationToken>,
    pub shutdown_strategy: ShutdownStrategy,
    pub channel_buffer_size: usize,
}

impl PipelineBuilder {
    /// Creates a new `PipelineBuilder` with empty collections for datasources,
    /// pipes, and metrics.
    ///
    /// This method initializes a `PipelineBuilder` instance, allowing you to
    /// configure each component of a `Pipeline` before building it. The
    /// builder pattern offers flexibility in adding data sources, account
    /// and transaction handling pipes, deletion processing, and metrics
    /// collection features.
    ///
    /// # Example
    ///
    /// ```rust
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new();
    /// ```
    pub fn new() -> Self {
        log::trace!("PipelineBuilder::new()");
        Self::default()
    }

    /// Adds a datasource to the pipeline.
    ///
    /// The datasource is responsible for providing updates, such as account and
    /// transaction data, to the pipeline. Multiple datasources can be added
    /// to handle various types of updates.
    ///
    /// # Parameters
    ///
    /// - `datasource`: The data source to add, implementing the `Datasource`
    ///   trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .datasource(MyDatasource::new());
    /// ```
    pub fn datasource(mut self, datasource: impl Datasource + 'static) -> Self {
        log::trace!("datasource(self, datasource: {:?})", stringify!(datasource));
        self.datasources
            .push((DatasourceId::new_unique(), Arc::new(datasource)));
        self
    }

    /// Adds a datasource to the pipeline with a specific ID.
    ///
    /// This method allows you to assign a custom ID to a datasource, which is
    /// useful for filtering updates based on their source. The ID can be used
    /// with filters to selectively process updates from specific datasources.
    ///
    /// # Parameters
    ///
    /// - `datasource`: The data source to add, implementing the `Datasource`
    ///   trait
    /// - `id`: The `DatasourceId` to assign to this datasource
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::{pipeline::PipelineBuilder, datasource::DatasourceId};
    ///
    /// let mainnet_id = DatasourceId::new_named("mainnet");
    /// let builder = PipelineBuilder::new()
    ///     .datasource_with_id(mainnet_id, MyDatasource::new());
    /// ```
    pub fn datasource_with_id(
        mut self,
        datasource: impl Datasource + 'static,
        id: DatasourceId,
    ) -> Self {
        log::trace!(
            "datasource_with_id(self, id: {:?}, datasource: {:?})",
            id,
            stringify!(datasource)
        );
        self.datasources.push((id, Arc::new(datasource)));
        self
    }

    /// Sets the shutdown strategy for the pipeline.
    ///
    /// This method configures how the pipeline should handle shutdowns. The
    /// shutdown strategy defines whether the pipeline should terminate
    /// immediately or continue processing pending updates after terminating
    /// the data sources.
    ///
    /// # Parameters
    ///
    /// - `shutdown_strategy`: A variant of [`ShutdownStrategy`] that determines
    ///   how the pipeline should handle shutdowns.
    ///
    /// # Returns
    ///
    /// Returns `Self`, allowing for method chaining.
    ///
    /// # Notes
    ///
    /// - Use `ShutdownStrategy::Immediate` to stop the entire pipeline
    ///   instantly, including all active processing tasks.
    /// - Use `ShutdownStrategy::ProcessPending` (the default) to terminate data
    ///   sources first and allow the pipeline to finish processing any updates
    ///   that are still pending.
    pub fn shutdown_strategy(mut self, shutdown_strategy: ShutdownStrategy) -> Self {
        log::trace!("shutdown_strategy(self, shutdown_strategy: {shutdown_strategy:?})");
        self.shutdown_strategy = shutdown_strategy;
        self
    }

    /// Adds an account pipe to process account updates.
    ///
    /// Account pipes decode and process updates to accounts within the
    /// pipeline. This method requires both an `AccountDecoder` and a
    /// `Processor` to handle decoded account data.
    ///
    /// # Parameters
    ///
    /// - `decoder`: An `AccountDecoder` that decodes the account data.
    /// - `processor`: A `Processor` that processes the decoded account data.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .account(MyAccountDecoder, MyAccountProcessor);
    /// ```
    pub fn account<T: Send + Sync + 'static>(
        mut self,
        decoder: impl for<'a> AccountDecoder<'a, AccountType = T> + Send + Sync + 'static,
        processor: impl Processor<InputType = AccountProcessorInputType<T>> + Send + Sync + 'static,
    ) -> Self {
        log::trace!(
            "account(self, decoder: {:?}, processor: {:?})",
            stringify!(decoder),
            stringify!(processor)
        );
        self.account_pipes.push(Box::new(AccountPipe {
            decoder: Box::new(decoder),
            processor: Box::new(processor),
            filters: vec![],
        }));
        self
    }

    /// Adds an account pipe with filters to process account updates selectively.
    ///
    /// This method creates an account pipe that only processes updates that pass
    /// all the specified filters. Filters can be used to selectively process
    /// updates based on criteria such as datasource ID, account properties, or
    /// other custom logic.
    ///
    /// # Parameters
    ///
    /// - `decoder`: An `AccountDecoder` that decodes the account data
    /// - `processor`: A `Processor` that processes the decoded account data
    /// - `filters`: A collection of filters that determine which account updates
    ///   should be processed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::{
    ///     pipeline::PipelineBuilder,
    ///     datasource::DatasourceId,
    ///     filter::DatasourceFilter,
    /// };
    ///
    /// let mainnet_id = DatasourceId::new_named("mainnet");
    /// let filter = DatasourceFilter::new(mainnet_id);
    /// let filters = vec![Box::new(filter) as Box<dyn carbon_core::filter::Filter>];
    ///
    /// let builder = PipelineBuilder::new()
    ///     .account_with_filters(MyAccountDecoder, MyAccountProcessor, filters);
    /// ```
    pub fn account_with_filters<T: Send + Sync + 'static>(
        mut self,
        decoder: impl for<'a> AccountDecoder<'a, AccountType = T> + Send + Sync + 'static,
        processor: impl Processor<InputType = AccountProcessorInputType<T>> + Send + Sync + 'static,
        filters: Vec<Box<dyn Filter + Send + Sync + 'static>>,
    ) -> Self {
        log::trace!(
            "account_with_filters(self, decoder: {:?}, processor: {:?}, filters: {:?})",
            stringify!(decoder),
            stringify!(processor),
            stringify!(filters)
        );
        self.account_pipes.push(Box::new(AccountPipe {
            decoder: Box::new(decoder),
            processor: Box::new(processor),
            filters,
        }));
        self
    }

    /// Adds an account deletion pipe to handle account deletion events.
    ///
    /// Account deletion pipes process deletions of accounts, with a `Processor`
    /// to handle the deletion events as they occur.
    ///
    /// # Parameters
    ///
    /// - `processor`: A `Processor` that processes account deletion events.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .account_deletions(MyAccountDeletionProcessor);
    /// ```
    pub fn account_deletions(
        mut self,
        processor: impl Processor<InputType = AccountDeletion> + Send + Sync + 'static,
    ) -> Self {
        log::trace!(
            "account_deletions(self, processor: {:?})",
            stringify!(processor)
        );
        self.account_deletion_pipes
            .push(Box::new(AccountDeletionPipe {
                processor: Box::new(processor),
                filters: vec![],
            }));
        self
    }

    /// Adds an account deletion pipe with filters to handle account deletion events selectively.
    ///
    /// This method creates an account deletion pipe that only processes deletion
    /// events that pass all the specified filters. Filters can be used to
    /// selectively process deletions based on criteria such as datasource ID or
    /// other custom logic.
    ///
    /// # Parameters
    ///
    /// - `processor`: A `Processor` that processes account deletion events
    /// - `filters`: A collection of filters that determine which account deletion
    ///   events should be processed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::{
    ///     pipeline::PipelineBuilder,
    ///     datasource::DatasourceId,
    ///     filter::DatasourceFilter,
    /// };
    ///
    /// let mainnet_id = DatasourceId::new_named("mainnet");
    /// let filter = DatasourceFilter::new(mainnet_id);
    /// let filters = vec![Box::new(filter) as Box<dyn carbon_core::filter::Filter>];
    ///
    /// let builder = PipelineBuilder::new()
    ///     .account_deletions_with_filters(MyAccountDeletionProcessor, filters);
    /// ```
    pub fn account_deletions_with_filters(
        mut self,
        processor: impl Processor<InputType = AccountDeletion> + Send + Sync + 'static,
        filters: Vec<Box<dyn Filter + Send + Sync + 'static>>,
    ) -> Self {
        log::trace!(
            "account_deletions_with_filters(self, processor: {:?}, filters: {:?})",
            stringify!(processor),
            stringify!(filters)
        );
        self.account_deletion_pipes
            .push(Box::new(AccountDeletionPipe {
                processor: Box::new(processor),
                filters,
            }));
        self
    }

    /// Adds a block details pipe to handle block details updates.
    ///
    /// Block details pipes process updates related to block metadata, such as
    /// slot, block hash, and rewards, with a `Processor` to handle the updates.
    ///
    /// # Parameters
    ///
    /// - `processor`: A `Processor` that processes block details updates.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .block_details(MyBlockDetailsProcessor);
    /// ```
    pub fn block_details(
        mut self,
        processor: impl Processor<InputType = BlockDetails> + Send + Sync + 'static,
    ) -> Self {
        log::trace!(
            "block_details(self, processor: {:?})",
            stringify!(processor)
        );
        self.block_details_pipes.push(Box::new(BlockDetailsPipe {
            processor: Box::new(processor),
            filters: vec![],
        }));
        self
    }

    /// Adds a block details pipe with filters to handle block details updates selectively.
    ///
    /// This method creates a block details pipe that only processes updates that
    /// pass all the specified filters. Filters can be used to selectively process
    /// block details updates based on criteria such as datasource ID, block height,
    /// or other custom logic.
    ///
    /// # Parameters
    ///
    /// - `processor`: A `Processor` that processes block details updates
    /// - `filters`: A collection of filters that determine which block details
    ///   updates should be processed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::{
    ///     pipeline::PipelineBuilder,
    ///     datasource::DatasourceId,
    ///     filter::DatasourceFilter,
    /// };
    ///
    /// let mainnet_id = DatasourceId::new_named("mainnet");
    /// let filter = DatasourceFilter::new(mainnet_id);
    /// let filters = vec![Box::new(filter) as Box<dyn carbon_core::filter::Filter>];
    ///
    /// let builder = PipelineBuilder::new()
    ///     .block_details_with_filters(MyBlockDetailsProcessor, filters);
    /// ```
    pub fn block_details_with_filters(
        mut self,
        processor: impl Processor<InputType = BlockDetails> + Send + Sync + 'static,
        filters: Vec<Box<dyn Filter + Send + Sync + 'static>>,
    ) -> Self {
        log::trace!(
            "block_details_with_filters(self, processor: {:?}, filters: {:?})",
            stringify!(processor),
            stringify!(filters)
        );
        self.block_details_pipes.push(Box::new(BlockDetailsPipe {
            processor: Box::new(processor),
            filters,
        }));
        self
    }

    /// Adds an instruction pipe to process instructions within transactions.
    ///
    /// Instruction pipes decode and process individual instructions,
    /// enabling specialized handling of various instruction types.
    ///
    /// # Parameters
    ///
    /// - `decoder`: An `InstructionDecoder` for decoding instructions from
    ///   transaction data.
    /// - `processor`: A `Processor` that processes decoded instruction data.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .instruction(MyDecoder, MyInstructionProcessor);
    /// ```
    pub fn instruction<T: Send + Sync + 'static>(
        mut self,
        decoder: impl for<'a> InstructionDecoder<'a, InstructionType = T> + Send + Sync + 'static,
        processor: impl Processor<InputType = InstructionProcessorInputType<T>> + Send + Sync + 'static,
    ) -> Self {
        log::trace!(
            "instruction(self, decoder: {:?}, processor: {:?})",
            stringify!(decoder),
            stringify!(processor)
        );
        self.instruction_pipes.push(Box::new(InstructionPipe {
            decoder: Box::new(decoder),
            processor: Box::new(processor),
            filters: vec![],
        }));
        self
    }

    /// Adds an instruction pipe with filters to process instructions selectively.
    ///
    /// This method creates an instruction pipe that only processes instructions
    /// that pass all the specified filters. Filters can be used to selectively
    /// process instructions based on criteria such as datasource ID, instruction
    /// type, or other custom logic.
    ///
    /// # Parameters
    ///
    /// - `decoder`: An `InstructionDecoder` for decoding instructions from
    ///   transaction data
    /// - `processor`: A `Processor` that processes decoded instruction data
    /// - `filters`: A collection of filters that determine which instructions
    ///   should be processed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::{
    ///     pipeline::PipelineBuilder,
    ///     datasource::DatasourceId,
    ///     filter::DatasourceFilter,
    /// };
    ///
    /// let mainnet_id = DatasourceId::new_named("mainnet");
    /// let filter = DatasourceFilter::new(mainnet_id);
    /// let filters = vec![Box::new(filter) as Box<dyn carbon_core::filter::Filter>];
    ///
    /// let builder = PipelineBuilder::new()
    ///     .instruction_with_filters(MyDecoder, MyInstructionProcessor, filters);
    /// ```
    pub fn instruction_with_filters<T: Send + Sync + 'static>(
        mut self,
        decoder: impl for<'a> InstructionDecoder<'a, InstructionType = T> + Send + Sync + 'static,
        processor: impl Processor<InputType = InstructionProcessorInputType<T>> + Send + Sync + 'static,
        filters: Vec<Box<dyn Filter + Send + Sync + 'static>>,
    ) -> Self {
        log::trace!(
            "instruction_with_filters(self, decoder: {:?}, processor: {:?}, filters: {:?})",
            stringify!(decoder),
            stringify!(processor),
            stringify!(filters)
        );
        self.instruction_pipes.push(Box::new(InstructionPipe {
            decoder: Box::new(decoder),
            processor: Box::new(processor),
            filters,
        }));
        self
    }

    /// Adds a transaction pipe for processing full transaction data.
    ///
    /// This method requires a transaction schema for decoding and a `Processor`
    /// to handle the processed transaction data.
    ///
    /// # Parameters
    ///
    /// - `schema`: A `TransactionSchema` used to match and interpret
    ///   transaction data.
    /// - `processor`: A `Processor` that processes the decoded transaction
    ///   data.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .transaction(MY_SCHEMA.clone(), MyTransactionProcessor);
    /// ```
    pub fn transaction<T, U>(
        mut self,
        processor: impl Processor<InputType = TransactionProcessorInputType<T, U>>
            + Send
            + Sync
            + 'static,
        schema: Option<TransactionSchema<T>>,
    ) -> Self
    where
        T: InstructionDecoderCollection + 'static,
        U: DeserializeOwned + Send + Sync + 'static,
    {
        log::trace!(
            "transaction(self, schema: {:?}, processor: {:?})",
            stringify!(schema),
            stringify!(processor)
        );
        self.transaction_pipes
            .push(Box::new(TransactionPipe::<T, U>::new(
                schema,
                processor,
                vec![],
            )));
        self
    }

    /// Adds a transaction pipe with filters for processing full transaction data selectively.
    ///
    /// This method creates a transaction pipe that only processes transactions
    /// that pass all the specified filters. Filters can be used to selectively
    /// process transactions based on criteria such as datasource ID, transaction
    /// type, or other custom logic.
    ///
    /// # Parameters
    ///
    /// - `processor`: A `Processor` that processes the decoded transaction data
    /// - `schema`: A `TransactionSchema` used to match and interpret
    ///   transaction data
    /// - `filters`: A collection of filters that determine which transactions
    ///   should be processed
    ///
    /// # Example
    ///
    /// ```ignore
    /// use carbon_core::{
    ///     pipeline::PipelineBuilder,
    ///     datasource::DatasourceId,
    ///     filter::DatasourceFilter,
    /// };
    ///
    /// let mainnet_id = DatasourceId::new_named("mainnet");
    /// let filter = DatasourceFilter::new(mainnet_id);
    /// let filters = vec![Box::new(filter) as Box<dyn carbon_core::filter::Filter>];
    ///
    /// let builder = PipelineBuilder::new()
    ///     .transaction_with_filters(MyTransactionProcessor, MY_SCHEMA.clone(), filters);
    /// ```
    pub fn transaction_with_filters<T, U>(
        mut self,
        processor: impl Processor<InputType = TransactionProcessorInputType<T, U>>
            + Send
            + Sync
            + 'static,
        schema: Option<TransactionSchema<T>>,
        filters: Vec<Box<dyn Filter + Send + Sync + 'static>>,
    ) -> Self
    where
        T: InstructionDecoderCollection + 'static,
        U: DeserializeOwned + Send + Sync + 'static,
    {
        log::trace!(
            "transaction_with_filters(self, schema: {:?}, processor: {:?}, filters: {:?})",
            stringify!(schema),
            stringify!(processor),
            stringify!(filters)
        );
        self.transaction_pipes
            .push(Box::new(TransactionPipe::<T, U>::new(
                schema, processor, filters,
            )));
        self
    }

    /// Adds a metrics component to the pipeline for performance tracking.
    ///
    /// This component collects and reports on pipeline metrics, providing
    /// insights into performance and operational statistics.
    ///
    /// # Parameters
    ///
    /// - `metrics`: An instance of a `Metrics` implementation, used to gather
    ///   and report metrics.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::sync::Arc;
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .metrics(Arc::new(LogMetrics::new()));
    /// ```
    pub fn metrics(mut self, metrics: Arc<dyn Metrics>) -> Self {
        log::trace!("metrics(self, metrics: {:?})", stringify!(metrics));
        self.metrics.metrics.push(metrics);
        self
    }

    /// Sets the interval for flushing metrics data.
    ///
    /// This value defines the frequency, in seconds, at which metrics data is
    /// flushed from memory. If not set, a default interval is used.
    ///
    /// # Parameters
    ///
    /// - `interval`: The flush interval for metrics, in seconds.
    ///
    /// # Example
    ///
    /// ```rust
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .metrics_flush_interval(60);
    /// ```
    pub fn metrics_flush_interval(mut self, interval: u64) -> Self {
        log::trace!("metrics_flush_interval(self, interval: {interval:?})");
        self.metrics_flush_interval = Some(interval);
        self
    }

    /// Sets the cancellation token for cancelling datasource on demand.
    ///
    /// This value is used to cancel datasource on demand.
    /// If not set, a default `CancellationToken` is used.
    ///
    /// # Parameters
    ///
    /// - `cancellation_token`: An instance of `CancellationToken`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use carbon_core::pipeline::PipelineBuilder;
    /// use tokio_util::sync::CancellationToken;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .datasource_cancellation_token(CancellationToken::new());
    /// ```
    pub fn datasource_cancellation_token(mut self, cancellation_token: CancellationToken) -> Self {
        log::trace!(
            "datasource_cancellation_token(self, cancellation_token: {cancellation_token:?})"
        );
        self.datasource_cancellation_token = Some(cancellation_token);
        self
    }

    /// Sets the size of the channel buffer for the pipeline.
    ///
    /// This value defines the maximum number of updates that can be queued in
    /// the pipeline's channel buffer. If not set, a default size of 10_000
    /// will be used.
    ///
    /// # Parameters
    ///
    /// - `size`: The size of the channel buffer for the pipeline.
    ///
    /// # Example
    ///
    /// ```rust
    /// use carbon_core::pipeline::PipelineBuilder;
    ///
    /// let builder = PipelineBuilder::new()
    ///     .channel_buffer_size(1000);
    /// ```
    pub fn channel_buffer_size(mut self, size: usize) -> Self {
        log::trace!("channel_buffer_size(self, size: {size:?})");
        self.channel_buffer_size = size;
        self
    }

    /// Builds and returns a `Pipeline` configured with the specified
    /// components.
    ///
    /// After configuring the `PipelineBuilder` with data sources, pipes, and
    /// metrics, call this method to create the final `Pipeline` instance
    /// ready for operation.
    ///
    /// # Returns
    ///
    /// Returns a `CarbonResult<Pipeline>` containing the configured `Pipeline`,
    /// or an error if any part of the configuration is invalid.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use std::sync::Arc;
    ///
    /// carbon_core::pipeline::Pipeline::builder()
    /// .datasource(transaction_crawler)
    /// .metrics(Arc::new(LogMetrics::new()))
    /// .metrics(Arc::new(PrometheusMetrics::new()))
    /// .instruction(
    ///    TestProgramDecoder,
    ///    TestProgramProcessor
    /// )
    /// .account(
    ///     TestProgramDecoder,
    ///     TestProgramAccountProcessor
    /// )
    /// .transaction(TEST_SCHEMA.clone(), TestProgramTransactionProcessor)
    /// .account_deletions(TestProgramAccountDeletionProcessor)
    /// .channel_buffer_size(1000)
    /// .build()?;
    ///
    ///  Ok(())
    /// ```
    pub fn build(self) -> CarbonResult<Pipeline> {
        log::trace!("build(self)");
        Ok(Pipeline {
            datasources: self.datasources,
            account_pipes: self.account_pipes,
            account_deletion_pipes: self.account_deletion_pipes,
            block_details_pipes: self.block_details_pipes,
            instruction_pipes: self.instruction_pipes,
            transaction_pipes: self.transaction_pipes,
            shutdown_strategy: self.shutdown_strategy,
            metrics: Arc::new(self.metrics),
            metrics_flush_interval: self.metrics_flush_interval,
            datasource_cancellation_token: self.datasource_cancellation_token,
            channel_buffer_size: self.channel_buffer_size,
        })
    }
}
