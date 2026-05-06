use std::marker::PhantomData;

use solana_instruction::AccountMeta;

use crate::{
    account::{AccountMetadata, AccountProcessorInputType, DecodedAccount},
    clickhouse::{
        metrics::{record_buffered_rows, ClickHouseMetricsFamily},
        rows::{ClickHouseRow, ClickHouseRowContext, ClickHouseRows},
        writer::{ClickHouseBatchWriter, ClickHouseBufferOutcome},
        ClickHouseConfig,
    },
    error::CarbonResult,
    instruction::{InstructionMetadata, InstructionProcessorInputType},
    processor::Processor,
};

pub use crate::clickhouse::metrics::register_clickhouse_metrics;

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
            writer: ClickHouseBatchWriter::<R>::new_with_metrics(
                config,
                ClickHouseMetricsFamily::Instructions,
            ),
            _phantom: PhantomData,
        }
    }

    pub async fn flush(&mut self) -> CarbonResult<usize> {
        self.writer.flush().await.map(|outcome| outcome.rows)
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
            match self.writer.buffer_row(row).await {
                Ok(ClickHouseBufferOutcome::Buffered { buffered_rows }) => {
                    record_buffered_rows(ClickHouseMetricsFamily::Instructions, buffered_rows);
                }
                Ok(ClickHouseBufferOutcome::Flushed(_)) => {}
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }

    async fn finalize(&mut self) -> CarbonResult<()> {
        self.writer.shutdown().await.map(|_| ())
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
            writer: ClickHouseBatchWriter::<R>::new_with_metrics(
                config,
                ClickHouseMetricsFamily::Accounts,
            ),
            _phantom: PhantomData,
        }
    }

    pub async fn flush(&mut self) -> CarbonResult<usize> {
        self.writer.flush().await.map(|outcome| outcome.rows)
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
            match self.writer.buffer_row(row).await {
                Ok(ClickHouseBufferOutcome::Buffered { buffered_rows }) => {
                    record_buffered_rows(ClickHouseMetricsFamily::Accounts, buffered_rows);
                }
                Ok(ClickHouseBufferOutcome::Flushed(_)) => {}
                Err(error) => return Err(error),
            }
        }

        Ok(())
    }

    async fn finalize(&mut self) -> CarbonResult<()> {
        self.writer.shutdown().await.map(|_| ())
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
