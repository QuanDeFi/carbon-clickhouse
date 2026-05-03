use std::{
    marker::PhantomData,
    sync::{LazyLock, Once},
    time::Instant,
};

use solana_instruction::AccountMeta;

use crate::{
    account::{AccountMetadata, AccountProcessorInputType, DecodedAccount},
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

static CLICKHOUSE_ACCOUNTS_INSERTED: Counter = Counter::new(
    "clickhouse.accounts.inserted",
    "Total number of ClickHouse account rows successfully inserted",
);

static CLICKHOUSE_ACCOUNTS_FAILED: Counter = Counter::new(
    "clickhouse.accounts.failed",
    "Total number of ClickHouse account rows in failed batches",
);

static CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS: Gauge = Gauge::new(
    "clickhouse.accounts.buffered_rows",
    "Current number of account rows buffered in the ClickHouse sink",
);

static CLICKHOUSE_ACCOUNTS_FLUSH_BATCHES: Counter = Counter::new(
    "clickhouse.accounts.flush.batches",
    "Total number of successful ClickHouse account flush batches",
);

static CLICKHOUSE_ACCOUNTS_FLUSH_FAILED_BATCHES: Counter = Counter::new(
    "clickhouse.accounts.flush.failed_batches",
    "Total number of failed ClickHouse account flush batches",
);

static CLICKHOUSE_ACCOUNTS_FLUSH_DURATION_MILLIS: LazyLock<Histogram> = LazyLock::new(|| {
    Histogram::new(
        "clickhouse.accounts.flush.duration_milliseconds",
        "Duration of ClickHouse account flush operations in milliseconds",
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
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_INSERTED);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FAILED);
        registry.register_gauge(&CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FLUSH_BATCHES);
        registry.register_counter(&CLICKHOUSE_ACCOUNTS_FLUSH_FAILED_BATCHES);
        registry.register_histogram(&CLICKHOUSE_ACCOUNTS_FLUSH_DURATION_MILLIS);
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

pub struct ClickHouseAccountProcessor<T, W, R>
where
    R: ClickHouseRow,
    W: ClickHouseRows<R>,
{
    row_context: ClickHouseRowContext,
    writer: ClickHouseBatchWriter<R>,
    _phantom: PhantomData<(T, W)>,
}

impl<T, W, R> ClickHouseAccountProcessor<T, W, R>
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
            CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS.set(0.0);
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
            CLICKHOUSE_ACCOUNTS_INSERTED.inc_by(outcome.rows as u64);
            CLICKHOUSE_ACCOUNTS_FLUSH_BATCHES.inc();
            CLICKHOUSE_ACCOUNTS_FLUSH_DURATION_MILLIS.record(duration_milliseconds);
        }
        CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS.set(outcome.buffered_rows as f64);
    }

    fn record_failed_flush(&self, buffered_rows: usize) {
        CLICKHOUSE_ACCOUNTS_FAILED.inc_by(buffered_rows as u64);
        CLICKHOUSE_ACCOUNTS_FLUSH_FAILED_BATCHES.inc();
        CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS.set(buffered_rows as f64);
    }
}

impl<T, W, R> Processor<AccountProcessorInputType<'_, T>> for ClickHouseAccountProcessor<T, W, R>
where
    T: Clone + Send + Sync + 'static,
    R: ClickHouseRow,
    W: ClickHouseRows<R> + From<(DecodedAccount<T>, AccountMetadata)> + Send + Sync + 'static,
{
    async fn process(&mut self, input: &AccountProcessorInputType<'_, T>) -> CarbonResult<()> {
        let wrapper = W::from((input.decoded_account.clone(), input.metadata.clone()));
        let rows = wrapper.clickhouse_rows(&self.row_context);

        if rows.is_empty() {
            return Ok(());
        }

        for row in rows {
            let start = Instant::now();
            match self.writer.buffer_row(row).await {
                Ok(ClickHouseBufferOutcome::Buffered { buffered_rows }) => {
                    CLICKHOUSE_ACCOUNTS_BUFFERED_ROWS.set(buffered_rows as f64);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        clickhouse::{rows::ClickHouseTable, ClickHouseConfig},
        metrics::MetricsRegistry,
    };
    use solana_pubkey::Pubkey;
    use std::time::Duration;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    #[derive(Debug, Clone)]
    struct DummyInstruction;

    #[derive(Debug, Clone)]
    struct DummyAccount;

    #[derive(Debug, Clone, serde::Serialize)]
    struct DummyRow {
        table: &'static str,
        partition: String,
    }

    impl ClickHouseTable for DummyRow {
        fn table() -> &'static str {
            "dummy_table"
        }

        fn columns() -> Vec<&'static str> {
            vec!["partition"]
        }

        fn create_table_sql(table_name: &str) -> String {
            format!(
                "CREATE TABLE IF NOT EXISTS {table_name} (partition String) ENGINE = MergeTree ORDER BY partition"
            )
        }
    }

    impl ClickHouseRow for DummyRow {
        fn table_name(&self) -> &'static str {
            self.table
        }

        fn partition_key(&self) -> String {
            self.partition.clone()
        }
    }

    struct EmptyInstructionWrapper;

    impl From<(DummyInstruction, InstructionMetadata, Vec<AccountMeta>)> for EmptyInstructionWrapper {
        fn from(_: (DummyInstruction, InstructionMetadata, Vec<AccountMeta>)) -> Self {
            Self
        }
    }

    impl ClickHouseRows<DummyRow> for EmptyInstructionWrapper {
        fn clickhouse_rows(&self, _context: &ClickHouseRowContext) -> Vec<DummyRow> {
            Vec::new()
        }
    }

    struct AccountWrapper(DecodedAccount<DummyAccount>, AccountMetadata);

    impl From<(DecodedAccount<DummyAccount>, AccountMetadata)> for AccountWrapper {
        fn from(value: (DecodedAccount<DummyAccount>, AccountMetadata)) -> Self {
            Self(value.0, value.1)
        }
    }

    impl ClickHouseRows<DummyRow> for AccountWrapper {
        fn clickhouse_rows(&self, _context: &ClickHouseRowContext) -> Vec<DummyRow> {
            let Self(_decoded_account, metadata) = self;
            vec![DummyRow {
                table: "dummy_accounts",
                partition: (metadata.slot / 1_000_000).to_string(),
            }]
        }
    }

    struct EmptyAccountWrapper;

    impl From<(DecodedAccount<DummyAccount>, AccountMetadata)> for EmptyAccountWrapper {
        fn from(_: (DecodedAccount<DummyAccount>, AccountMetadata)) -> Self {
            Self
        }
    }

    impl ClickHouseRows<DummyRow> for EmptyAccountWrapper {
        fn clickhouse_rows(&self, _context: &ClickHouseRowContext) -> Vec<DummyRow> {
            Vec::new()
        }
    }

    fn config() -> ClickHouseConfig {
        config_with_endpoint("http://localhost:8123".to_string())
    }

    fn config_with_endpoint(endpoint: String) -> ClickHouseConfig {
        ClickHouseConfig::new(
            endpoint,
            "default".to_string(),
            None,
            None,
            "dummy_table".to_string(),
            "source".to_string(),
            "live".to_string(),
            "v1".to_string(),
            100,
            Duration::from_secs(60),
        )
    }

    async fn start_clickhouse_server(
        expected_requests: usize,
    ) -> (String, tokio::task::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();

            for _ in 0..expected_requests {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buffer = vec![0u8; 8192];
                let n = socket.read(&mut buffer).await.unwrap();
                requests.push(String::from_utf8_lossy(&buffer[..n]).to_string());
                socket
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                    .await
                    .unwrap();
            }

            requests
        });

        (endpoint, handle)
    }

    fn counter_value(name: &str) -> u64 {
        MetricsRegistry::global()
            .snapshot()
            .counters
            .into_iter()
            .filter(|(metric_name, _, _)| *metric_name == name)
            .map(|(_, _, value)| value)
            .next()
            .unwrap_or_default()
    }

    fn gauge_value(name: &str) -> f64 {
        MetricsRegistry::global()
            .snapshot()
            .gauges
            .into_iter()
            .filter(|(metric_name, _, _)| *metric_name == name)
            .map(|(_, _, value)| value)
            .next()
            .unwrap_or_default()
    }

    fn account_metadata() -> AccountMetadata {
        AccountMetadata {
            slot: 2_000_000,
            pubkey: Pubkey::new_unique(),
            transaction_signature: None,
        }
    }

    fn decoded_account() -> DecodedAccount<DummyAccount> {
        DecodedAccount {
            lamports: 1,
            data: DummyAccount,
            owner: Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        }
    }

    #[tokio::test]
    async fn instruction_processor_finalize_drains_empty_writer() {
        let mut processor = ClickHouseInstructionProcessor::<
            DummyInstruction,
            EmptyInstructionWrapper,
            DummyRow,
        >::new(config());

        processor.finalize().await.unwrap();
        assert_eq!(processor.writer.buffered_rows(), 0);
    }

    #[tokio::test]
    async fn account_processor_buffers_rows() {
        let mut processor =
            ClickHouseAccountProcessor::<DummyAccount, AccountWrapper, DummyRow>::new(config());
        let metadata = account_metadata();
        let decoded_account = decoded_account();
        let raw_account = solana_account::Account::default();
        let input = AccountProcessorInputType {
            metadata: &metadata,
            decoded_account: &decoded_account,
            raw_account: &raw_account,
        };

        processor.process(&input).await.unwrap();

        assert_eq!(processor.writer.buffered_rows(), 1);
    }

    #[tokio::test]
    async fn account_processor_finalize_drains_empty_writer() {
        let mut processor =
            ClickHouseAccountProcessor::<DummyAccount, EmptyAccountWrapper, DummyRow>::new(config());

        processor.finalize().await.unwrap();
        assert_eq!(processor.writer.buffered_rows(), 0);
    }

    #[tokio::test]
    async fn account_processor_finalize_flushes_buffered_rows_and_records_metrics() {
        let (endpoint, server) = start_clickhouse_server(1).await;
        let before_inserted = counter_value("clickhouse.accounts.inserted");
        let mut processor =
            ClickHouseAccountProcessor::<DummyAccount, AccountWrapper, DummyRow>::new(
                config_with_endpoint(endpoint),
            );
        let metadata = account_metadata();
        let decoded_account = decoded_account();
        let raw_account = solana_account::Account::default();
        let input = AccountProcessorInputType {
            metadata: &metadata,
            decoded_account: &decoded_account,
            raw_account: &raw_account,
        };

        processor.process(&input).await.unwrap();
        processor.finalize().await.unwrap();

        let requests = server.await.unwrap().join("\n");
        assert_eq!(processor.writer.buffered_rows(), 0);
        assert!(requests.contains("dummy_accounts"));
        assert!(counter_value("clickhouse.accounts.inserted") > before_inserted);
        assert_eq!(gauge_value("clickhouse.accounts.buffered_rows"), 0.0);
    }
}
