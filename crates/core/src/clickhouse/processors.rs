use std::sync::Arc;

use solana_instruction::AccountMeta;

use crate::{
    clickhouse::{rows::{ClickHouseRow, ClickHouseRows}, writer::{ClickHouseBatchWriter, ClickHouseBufferOutcome}, ClickHouseConfig},
    error::CarbonResult,
    instruction::{InstructionMetadata, InstructionProcessorInputType},
    metrics::MetricsCollection,
};

pub struct ClickHouseInstructionProcessor<T, W, R>
where
    R: ClickHouseRow,
    W: ClickHouseRows<R>,
{
    config: ClickHouseConfig,
    writer: ClickHouseBatchWriter<R>,
    _phantom: std::marker::PhantomData<(T, W)>,
}

impl<T, W, R> ClickHouseInstructionProcessor<T, W, R>
where
    R: ClickHouseRow,
    W: ClickHouseRows<R>,
{
    pub fn new(config: ClickHouseConfig) -> Self {
        let writer = ClickHouseBatchWriter::new(config.clone());
        Self {
            config,
            writer,
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn flush(&mut self) -> CarbonResult<usize> {
        self.writer.flush().await
    }

    async fn record_successful_flush(
        &mut self,
        metrics: Arc<MetricsCollection>,
        rows: usize,
        duration_milliseconds: f64,
    ) -> CarbonResult<()> {
        metrics
            .increment_counter("clickhouse.instructions.inserted", rows as u64)
            .await?;
        metrics
            .increment_counter("clickhouse.instructions.flush.batches", 1)
            .await?;
        metrics
            .record_histogram(
                "clickhouse.instructions.flush.duration_milliseconds",
                duration_milliseconds,
            )
            .await?;
        metrics
            .update_gauge(
                "clickhouse.instructions.buffered_rows",
                self.writer.buffered_rows() as f64,
            )
            .await?;
        Ok(())
    }

    async fn record_failed_flush(
        &mut self,
        metrics: Arc<MetricsCollection>,
        rows: usize,
    ) -> CarbonResult<()> {
        metrics
            .increment_counter("clickhouse.instructions.failed", rows as u64)
            .await?;
        metrics
            .increment_counter("clickhouse.instructions.flush.failed_batches", 1)
            .await?;
        metrics
            .update_gauge(
                "clickhouse.instructions.buffered_rows",
                self.writer.buffered_rows() as f64,
            )
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<T, W, R> crate::processor::Processor for ClickHouseInstructionProcessor<T, W, R>
where
    T: Clone + Send + Sync + 'static,
    R: ClickHouseRow,
    W: ClickHouseRows<R> + From<(T, InstructionMetadata, Vec<AccountMeta>)> + Send + Sync + 'static,
{
    type InputType = InstructionProcessorInputType<T>;

    async fn process(
        &mut self,
        input: Self::InputType,
        metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let (metadata, decoded_instruction, _nested_instructions, _raw) = input;

        let wrapper = W::from((
            decoded_instruction.data,
            metadata,
            decoded_instruction.accounts,
        ));
        let rows = wrapper.clickhouse_rows(&self.config);

        if rows.is_empty() {
            return Ok(());
        }

        for row in rows {
            let start = std::time::Instant::now();
            match self.writer.buffer_row(row).await {
                Ok(ClickHouseBufferOutcome::Buffered) => {
                    metrics
                        .update_gauge(
                            "clickhouse.instructions.buffered_rows",
                            self.writer.buffered_rows() as f64,
                        )
                        .await?;
                }
                Ok(ClickHouseBufferOutcome::Flushed { rows }) => {
                    self.record_successful_flush(
                        metrics.clone(),
                        rows,
                        start.elapsed().as_millis() as f64,
                    )
                    .await?;
                }
                Err(e) => {
                    let buffered_rows = self.writer.buffered_rows();
                    self.record_failed_flush(metrics.clone(), buffered_rows).await?;
                    return Err(e);
                }
            }
        }

        Ok(())
    }

    async fn finalize(&mut self, metrics: Arc<MetricsCollection>) -> CarbonResult<()> {
        let start = std::time::Instant::now();
        match self.writer.flush().await {
            Ok(rows) => {
                if rows > 0 {
                    self.record_successful_flush(
                        metrics,
                        rows,
                        start.elapsed().as_millis() as f64,
                    )
                    .await?;
                } else {
                    metrics
                        .update_gauge(
                            "clickhouse.instructions.buffered_rows",
                            self.writer.buffered_rows() as f64,
                        )
                        .await?;
                }
                Ok(())
            }
            Err(e) => {
                let buffered_rows = self.writer.buffered_rows();
                self.record_failed_flush(metrics, buffered_rows).await?;
                Err(e)
            }
        }
    }
}
