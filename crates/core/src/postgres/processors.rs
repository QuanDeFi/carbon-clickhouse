use solana_instruction::AccountMeta;
use std::sync::Arc;

use crate::{
    account::{AccountMetadata, AccountProcessorInputType},
    error::CarbonResult,
    instruction::{InstructionMetadata, InstructionProcessorInputType},
    metrics::MetricsCollection,
    postgres::{
        operations::Upsert,
        rows::{AccountRow, InstructionRow},
    },
};
const BUILD_INSTRUCTION_ROW_WRAPPER_STAGE: &str = "stage.build_instruction_row_wrapper.slot";
const UPSERT_INSTRUCTION_ROW_POSTGRES_STAGE: &str = "stage.upsert_instruction_row_postgres.slot";
const UPSERT_INSTRUCTION_ROW_POSTGRES_STAGE_EXIT: &str =
    "stage.upsert_instruction_row_postgres.exit.slot";

pub struct PostgresAccountProcessor<T, W> {
    pool: sqlx::PgPool,
    _phantom: std::marker::PhantomData<(T, W)>,
}

impl<T, W> PostgresAccountProcessor<T, W> {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T, W> crate::processor::Processor for PostgresAccountProcessor<T, W>
where
    T: Clone + Send + Sync + 'static,
    W: From<(T, AccountMetadata)> + Upsert + Send + 'static,
{
    type InputType = AccountProcessorInputType<T>;

    async fn process(
        &mut self,
        input: Self::InputType,
        metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (metadata, decoded_account, _raw) = input;

        let start = std::time::Instant::now();

        let wrapper = W::from((decoded_account.data, metadata));

        match wrapper.upsert(&self.pool).await {
            Ok(()) => {
                metrics
                    .increment_counter("postgres.accounts.upsert.upserted", 1)
                    .await?;
                metrics
                    .record_histogram(
                        "postgres.accounts.upsert.duration_milliseconds",
                        start.elapsed().as_millis() as f64,
                    )
                    .await?;
                Ok(())
            }
            Err(e) => {
                metrics
                    .increment_counter("postgres.accounts.upsert.failed", 1)
                    .await?;
                return Err(e);
            }
        }
    }
}

pub struct PostgresJsonAccountProcessor<T> {
    pool: sqlx::PgPool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> PostgresJsonAccountProcessor<T> {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T> crate::processor::Processor for PostgresJsonAccountProcessor<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + Unpin + 'static,
{
    type InputType = AccountProcessorInputType<T>;

    async fn process(
        &mut self,
        input: Self::InputType,
        metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (metadata, decoded_account, _raw) = input;

        let account_row = AccountRow::from_parts(decoded_account.data, metadata);

        let start = std::time::Instant::now();

        match account_row.upsert(&self.pool).await {
            Ok(()) => {
                metrics
                    .increment_counter("postgres.accounts.upsert.upserted", 1)
                    .await?;
                metrics
                    .record_histogram(
                        "postgres.accounts.upsert.duration_milliseconds",
                        start.elapsed().as_millis() as f64,
                    )
                    .await?;
                Ok(())
            }
            Err(e) => {
                metrics
                    .increment_counter("postgres.accounts.upsert.failed", 1)
                    .await?;
                return Err(e);
            }
        }
    }
}

pub struct PostgresInstructionProcessor<T, W> {
    pool: sqlx::PgPool,
    _phantom: std::marker::PhantomData<(T, W)>,
}

impl<T, W> PostgresInstructionProcessor<T, W> {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T, W> crate::processor::Processor for PostgresInstructionProcessor<T, W>
where
    T: Clone + Send + Sync + 'static,
    W: From<(T, InstructionMetadata, Vec<AccountMeta>)> + Upsert + Send + 'static,
{
    type InputType = InstructionProcessorInputType<T>;

    async fn process(
        &mut self,
        input: Self::InputType,
        metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (metadata, decoded_instruction, _nested_instructions, _raw) = input;
        let slot = metadata.transaction_metadata.slot;

        let start = std::time::Instant::now();

        let wrapper = W::from((
            decoded_instruction.data,
            metadata,
            decoded_instruction.accounts,
        ));
        metrics
            .update_gauge(BUILD_INSTRUCTION_ROW_WRAPPER_STAGE, slot as f64)
            .await?;
        metrics
            .increment_counter("postgres.instructions.row_wrapper.built", 1)
            .await?;

        metrics
            .update_gauge(UPSERT_INSTRUCTION_ROW_POSTGRES_STAGE, slot as f64)
            .await?;

        match wrapper.upsert(&self.pool).await {
            Ok(()) => {
                metrics
                    .increment_counter("postgres.instructions.upsert.upserted", 1)
                    .await?;
                metrics
                    .record_histogram(
                        "postgres.instructions.upsert.duration_milliseconds",
                        start.elapsed().as_millis() as f64,
                    )
                    .await?;
                metrics
                    .update_gauge(UPSERT_INSTRUCTION_ROW_POSTGRES_STAGE_EXIT, slot as f64)
                    .await?;
                Ok(())
            }
            Err(e) => {
                metrics
                    .increment_counter("postgres.instructions.upsert.failed", 1)
                    .await?;
                metrics
                    .update_gauge(UPSERT_INSTRUCTION_ROW_POSTGRES_STAGE_EXIT, slot as f64)
                    .await?;
                return Err(e);
            }
        }
    }
}

pub struct PostgresJsonInstructionProcessor<T> {
    pool: sqlx::PgPool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> PostgresJsonInstructionProcessor<T> {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self {
            pool,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<T> crate::processor::Processor for PostgresJsonInstructionProcessor<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + Unpin + 'static,
{
    type InputType = InstructionProcessorInputType<T>;

    async fn process(
        &mut self,
        input: Self::InputType,
        metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (metadata, decoded_instruction, _nested_instructions, _raw) = input;
        let slot = metadata.transaction_metadata.slot;

        let instruction_row = InstructionRow::from_parts(
            decoded_instruction.data,
            metadata,
            decoded_instruction.accounts,
        );
        metrics
            .update_gauge(BUILD_INSTRUCTION_ROW_WRAPPER_STAGE, slot as f64)
            .await?;
        metrics
            .increment_counter("postgres.instructions.row_wrapper.built", 1)
            .await?;

        let start = std::time::Instant::now();

        metrics
            .update_gauge(UPSERT_INSTRUCTION_ROW_POSTGRES_STAGE, slot as f64)
            .await?;

        match instruction_row.upsert(&self.pool).await {
            Ok(()) => {
                metrics
                    .increment_counter("postgres.instructions.upsert.upserted", 1)
                    .await?;
                metrics
                    .record_histogram(
                        "postgres.instructions.upsert.duration_milliseconds",
                        start.elapsed().as_millis() as f64,
                    )
                    .await?;
                metrics
                    .update_gauge(UPSERT_INSTRUCTION_ROW_POSTGRES_STAGE_EXIT, slot as f64)
                    .await?;
                Ok(())
            }
            Err(e) => {
                metrics
                    .increment_counter("postgres.instructions.upsert.failed", 1)
                    .await?;
                metrics
                    .update_gauge(UPSERT_INSTRUCTION_ROW_POSTGRES_STAGE_EXIT, slot as f64)
                    .await?;
                return Err(e);
            }
        }
    }
}
