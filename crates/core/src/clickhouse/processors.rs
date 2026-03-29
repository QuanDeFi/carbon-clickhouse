use std::{
    marker::PhantomData,
    sync::{LazyLock, Once},
    time::Instant,
};

use solana_instruction::AccountMeta;

use crate::{
    clickhouse::{
        rows::{ClickHouseRow, ClickHouseRowContext, ClickHouseRows},
        writer::{ClickHouseBatchWriter, ClickHouseBufferOutcome, ClickHouseFlushOutcome},
        ClickHouseConfig,
    },
    error::CarbonResult,
    instruction::{InstructionMetadata, InstructionProcessorInputType},
    metrics::{Counter, Gauge, Histogram, MetricsRegistry},
    processor::Processor,
};

static CLICKHOUSE_INSTRUCTIONS_INSERTED: Counter = Counter::new(
    "clickhouse.instructions.inserted",
    "Total number of ClickHouse instruction rows successfully inserted",
);

static CLICKHOUSE_INSTRUCTIONS_FAILED: Counter = Counter::new(
    "clickhouse.instructions.failed",
    "Total number of ClickHouse instruction rows in failed batches",
);

static CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS: Gauge = Gauge::new(
    "clickhouse.instructions.buffered_rows",
    "Current number of instruction rows buffered in the ClickHouse sink",
);

static CLICKHOUSE_INSTRUCTIONS_FLUSH_BATCHES: Counter = Counter::new(
    "clickhouse.instructions.flush.batches",
    "Total number of successful ClickHouse instruction flush batches",
);

static CLICKHOUSE_INSTRUCTIONS_FLUSH_FAILED_BATCHES: Counter = Counter::new(
    "clickhouse.instructions.flush.failed_batches",
    "Total number of failed ClickHouse instruction flush batches",
);

static CLICKHOUSE_INSTRUCTIONS_FLUSH_DURATION_MILLIS: LazyLock<Histogram> = LazyLock::new(|| {
    Histogram::new(
        "clickhouse.instructions.flush.duration_milliseconds",
        "Duration of ClickHouse instruction flush operations in milliseconds",
        vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0],
    )
});

static REGISTER_CLICKHOUSE_METRICS: Once = Once::new();

pub fn register_clickhouse_metrics() {
    REGISTER_CLICKHOUSE_METRICS.call_once(|| {
        let registry = MetricsRegistry::global();
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_INSERTED);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FAILED);
        registry.register_gauge(&CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FLUSH_BATCHES);
        registry.register_counter(&CLICKHOUSE_INSTRUCTIONS_FLUSH_FAILED_BATCHES);
        registry.register_histogram(&CLICKHOUSE_INSTRUCTIONS_FLUSH_DURATION_MILLIS);
    });
}

pub struct ClickHouseInstructionProcessor<T, W, R>
where
    R: ClickHouseRow,
    W: ClickHouseRows<R>,
{
    row_context: ClickHouseRowContext,
    writer: ClickHouseBatchWriter<R>,
    _phantom: PhantomData<(T, W)>,
}

impl<T, W, R> ClickHouseInstructionProcessor<T, W, R>
where
    R: ClickHouseRow,
    W: ClickHouseRows<R>,
{
    pub fn new(config: ClickHouseConfig) -> Self {
        register_clickhouse_metrics();
        Self {
            row_context: config.row_context(),
            writer: ClickHouseBatchWriter::new(config),
            _phantom: PhantomData,
        }
    }

    pub async fn flush(&mut self) -> CarbonResult<usize> {
        self.flush_internal().await.map(|outcome| outcome.rows)
    }

    async fn flush_internal(&mut self) -> CarbonResult<ClickHouseFlushOutcome> {
        if self.writer.buffered_rows() == 0 {
            CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS.set(0.0);
            return Ok(ClickHouseFlushOutcome {
                rows: 0,
                buffered_rows: 0,
            });
        }

        let start = Instant::now();
        match self.writer.flush().await {
            Ok(outcome) => {
                self.record_successful_flush(outcome, start.elapsed().as_millis() as f64);
                Ok(outcome)
            }
            Err(error) => {
                self.record_failed_flush(self.writer.buffered_rows());
                Err(error)
            }
        }
    }

    fn record_successful_flush(&self, outcome: ClickHouseFlushOutcome, duration_milliseconds: f64) {
        if outcome.rows > 0 {
            CLICKHOUSE_INSTRUCTIONS_INSERTED.inc_by(outcome.rows as u64);
            CLICKHOUSE_INSTRUCTIONS_FLUSH_BATCHES.inc();
            CLICKHOUSE_INSTRUCTIONS_FLUSH_DURATION_MILLIS.record(duration_milliseconds);
        }
        CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS.set(outcome.buffered_rows as f64);
    }

    fn record_failed_flush(&self, buffered_rows: usize) {
        CLICKHOUSE_INSTRUCTIONS_FAILED.inc_by(buffered_rows as u64);
        CLICKHOUSE_INSTRUCTIONS_FLUSH_FAILED_BATCHES.inc();
        CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS.set(buffered_rows as f64);
    }
}

impl<T, W, R> Processor<InstructionProcessorInputType<'_, T>>
    for ClickHouseInstructionProcessor<T, W, R>
where
    T: Clone + Send + Sync + 'static,
    R: ClickHouseRow,
    W: ClickHouseRows<R> + From<(T, InstructionMetadata, Vec<AccountMeta>)> + Send + Sync + 'static,
{
    async fn process(&mut self, input: &InstructionProcessorInputType<'_, T>) -> CarbonResult<()> {
        let wrapper = W::from((
            input.decoded_instruction.clone(),
            input.metadata.clone(),
            input.raw_instruction.accounts.clone(),
        ));
        let rows = wrapper.clickhouse_rows(&self.row_context);

        if rows.is_empty() {
            return Ok(());
        }

        for row in rows {
            let start = Instant::now();
            match self.writer.buffer_row(row).await {
                Ok(ClickHouseBufferOutcome::Buffered { buffered_rows }) => {
                    CLICKHOUSE_INSTRUCTIONS_BUFFERED_ROWS.set(buffered_rows as f64);
                }
                Ok(ClickHouseBufferOutcome::Flushed(outcome)) => {
                    self.record_successful_flush(outcome, start.elapsed().as_millis() as f64);
                }
                Err(error) => {
                    self.record_failed_flush(self.writer.buffered_rows());
                    return Err(error);
                }
            }
        }

        Ok(())
    }

    async fn finalize(&mut self) -> CarbonResult<()> {
        self.flush_internal().await.map(|_| ())
    }
}
