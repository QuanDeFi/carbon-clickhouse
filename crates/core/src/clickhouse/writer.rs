use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::{
    clickhouse::{
        config::{ClickHouseConfig, ClickHouseDeduplicationSettings, ClickHouseQuerySetting},
        http::{client_from_config, post_query_with_data, ClickHouseHttpError},
        metrics::{
            record_backpressure_rejected, record_buffer_state, record_failed_flush, record_retry,
            record_successful_flush, ClickHouseMetricsFamily,
        },
        rows::ClickHouseRow,
    },
    error::{CarbonResult, Error},
};

pub struct ClickHouseBatchWriter<R: ClickHouseRow> {
    config: ClickHouseConfig,
    client: reqwest::Client,
    state: Arc<Mutex<ClickHouseWriterState>>,
    shutdown_token: CancellationToken,
    background_task: Option<tokio::task::JoinHandle<()>>,
    metrics_family: ClickHouseMetricsFamily,
    _phantom: PhantomData<R>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ClickHouseBufferKey {
    table: &'static str,
    partition: String,
}

#[derive(Debug)]
struct ClickHouseBuffer {
    rows: Vec<String>,
    bytes: usize,
    last_flush: Instant,
}

#[derive(Debug)]
struct ClickHouseWriterState {
    buffers: HashMap<ClickHouseBufferKey, ClickHouseBuffer>,
    buffered_rows: usize,
    buffered_bytes: usize,
    background_error: Option<String>,
    shutdown: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickHouseBufferOutcome {
    Buffered { buffered_rows: usize },
    Flushed(ClickHouseFlushOutcome),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickHouseFlushOutcome {
    pub rows: usize,
    pub buffered_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseWriterSnapshot {
    pub buffered_rows: usize,
    pub buffered_bytes: usize,
    pub active_buffers: usize,
    pub background_error: Option<String>,
}

impl<R: ClickHouseRow> ClickHouseBatchWriter<R> {
    pub fn new(config: ClickHouseConfig) -> Self {
        Self::new_with_metrics(config, ClickHouseMetricsFamily::Instructions)
    }

    pub(crate) fn new_with_metrics(
        config: ClickHouseConfig,
        metrics_family: ClickHouseMetricsFamily,
    ) -> Self {
        Self {
            client: client_from_config(&config),
            config,
            state: Arc::new(Mutex::new(ClickHouseWriterState {
                buffers: HashMap::new(),
                buffered_rows: 0,
                buffered_bytes: 0,
                background_error: None,
                shutdown: false,
            })),
            shutdown_token: CancellationToken::new(),
            background_task: None,
            metrics_family,
            _phantom: PhantomData,
        }
    }

    pub fn buffered_rows(&self) -> usize {
        self.state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .buffered_rows
    }

    pub fn buffered_bytes(&self) -> usize {
        self.state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .buffered_bytes
    }

    pub fn snapshot(&self) -> ClickHouseWriterSnapshot {
        let state = self.state.lock().expect("ClickHouse writer state poisoned");
        ClickHouseWriterSnapshot {
            buffered_rows: state.buffered_rows,
            buffered_bytes: state.buffered_bytes,
            active_buffers: state.buffers.len(),
            background_error: state.background_error.clone(),
        }
    }

    #[cfg(test)]
    fn buffer_count(&self) -> usize {
        self.state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .buffers
            .len()
    }

    pub async fn buffer_row(&mut self, row: R) -> CarbonResult<ClickHouseBufferOutcome> {
        self.ensure_background_task();
        self.take_background_error()?;

        let serialized_row = row.to_json_line()?;
        let row_bytes = serialized_row.len() + 1;
        let key = ClickHouseBufferKey {
            table: row.table_name(),
            partition: row.partition_key(),
        };

        self.ensure_capacity(row_bytes).await?;

        let (buffered_rows, buffered_bytes, active_buffers) = {
            let mut state = self.state.lock().expect("ClickHouse writer state poisoned");
            if state.shutdown {
                return Err(Error::Custom(
                    "Cannot buffer rows after ClickHouse writer shutdown".to_string(),
                ));
            }

            let buffer = state
                .buffers
                .entry(key.clone())
                .or_insert_with(|| ClickHouseBuffer {
                    rows: Vec::new(),
                    bytes: 0,
                    last_flush: Instant::now(),
                });
            buffer.rows.push(serialized_row);
            buffer.bytes += row_bytes;
            state.buffered_rows += 1;
            state.buffered_bytes += row_bytes;
            (
                state.buffered_rows,
                state.buffered_bytes,
                state.buffers.len(),
            )
        };

        record_buffer_state(
            self.metrics_family,
            buffered_rows,
            buffered_bytes,
            active_buffers,
        );

        if self.buffer_should_flush(&key) {
            return self
                .flush_buffer(key)
                .await
                .map(ClickHouseBufferOutcome::Flushed);
        }

        Ok(ClickHouseBufferOutcome::Buffered { buffered_rows })
    }

    pub async fn flush(&mut self) -> CarbonResult<ClickHouseFlushOutcome> {
        self.ensure_background_task();
        let outcome = Self::flush_all_buffers(
            Arc::clone(&self.state),
            self.client.clone(),
            self.config.clone(),
            self.metrics_family,
        )
        .await?;

        if outcome.rows > 0 {
            self.clear_background_error();
        } else {
            self.take_background_error()?;
        }

        Ok(outcome)
    }

    pub async fn shutdown(&mut self) -> CarbonResult<ClickHouseFlushOutcome> {
        self.ensure_background_task();
        {
            let mut state = self.state.lock().expect("ClickHouse writer state poisoned");
            state.shutdown = true;
        }
        self.shutdown_token.cancel();

        if let Some(task) = self.background_task.take() {
            task.await.map_err(|err| {
                Error::Custom(format!("ClickHouse background flush failed: {err}"))
            })?;
        }

        self.flush().await
    }

    fn ensure_background_task(&mut self) {
        if self
            .state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .shutdown
        {
            return;
        }

        if self.background_task.is_some() {
            return;
        }

        let state = Arc::clone(&self.state);
        let client = self.client.clone();
        let config = self.config.clone();
        let metrics_family = self.metrics_family;
        let shutdown_token = self.shutdown_token.clone();
        let interval = nonzero_interval(config.batch_settings.flush_interval);

        self.background_task = Some(tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_token.cancelled() => break,
                    _ = tokio::time::sleep(interval) => {
                        if Self::is_shutdown(&state) {
                            break;
                        }

                        let _ = Self::flush_stale_buffers(
                            Arc::clone(&state),
                            client.clone(),
                            config.clone(),
                            metrics_family,
                        )
                        .await;
                    }
                }
            }
        }));
    }

    async fn flush_buffer(&self, key: ClickHouseBufferKey) -> CarbonResult<ClickHouseFlushOutcome> {
        Self::flush_buffer_key(
            Arc::clone(&self.state),
            self.client.clone(),
            self.config.clone(),
            self.metrics_family,
            key,
        )
        .await
    }

    async fn flush_all_buffers(
        state: Arc<Mutex<ClickHouseWriterState>>,
        client: reqwest::Client,
        config: ClickHouseConfig,
        metrics_family: ClickHouseMetricsFamily,
    ) -> CarbonResult<ClickHouseFlushOutcome> {
        let keys = {
            let state = state.lock().expect("ClickHouse writer state poisoned");
            state.buffers.keys().cloned().collect::<Vec<_>>()
        };

        Self::flush_keys(state, client, config, metrics_family, keys).await
    }

    async fn flush_stale_buffers(
        state: Arc<Mutex<ClickHouseWriterState>>,
        client: reqwest::Client,
        config: ClickHouseConfig,
        metrics_family: ClickHouseMetricsFamily,
    ) -> CarbonResult<ClickHouseFlushOutcome> {
        let now = Instant::now();
        let keys = {
            let state = state.lock().expect("ClickHouse writer state poisoned");
            state
                .buffers
                .iter()
                .filter_map(|(key, buffer)| {
                    if now.duration_since(buffer.last_flush) >= config.batch_settings.flush_interval
                    {
                        Some(key.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        Self::flush_keys(state, client, config, metrics_family, keys).await
    }

    async fn flush_keys(
        state: Arc<Mutex<ClickHouseWriterState>>,
        client: reqwest::Client,
        config: ClickHouseConfig,
        metrics_family: ClickHouseMetricsFamily,
        keys: Vec<ClickHouseBufferKey>,
    ) -> CarbonResult<ClickHouseFlushOutcome> {
        let mut flushed = 0usize;
        for key in keys {
            let outcome = Self::flush_buffer_key(
                Arc::clone(&state),
                client.clone(),
                config.clone(),
                metrics_family,
                key,
            )
            .await?;
            flushed += outcome.rows;
        }

        let buffered_rows = state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .buffered_rows;
        Ok(ClickHouseFlushOutcome {
            rows: flushed,
            buffered_rows,
        })
    }

    async fn flush_buffer_key(
        state: Arc<Mutex<ClickHouseWriterState>>,
        client: reqwest::Client,
        config: ClickHouseConfig,
        metrics_family: ClickHouseMetricsFamily,
        key: ClickHouseBufferKey,
    ) -> CarbonResult<ClickHouseFlushOutcome> {
        let Some(buffer) = ({
            let mut state = state.lock().expect("ClickHouse writer state poisoned");
            state.buffers.remove(&key)
        }) else {
            let buffered_rows = state
                .lock()
                .expect("ClickHouse writer state poisoned")
                .buffered_rows;
            return Ok(ClickHouseFlushOutcome {
                rows: 0,
                buffered_rows,
            });
        };

        if buffer.rows.is_empty() {
            let buffered_rows = state
                .lock()
                .expect("ClickHouse writer state poisoned")
                .buffered_rows;
            return Ok(ClickHouseFlushOutcome {
                rows: 0,
                buffered_rows,
            });
        }

        let rows_len = buffer.rows.len();
        let bytes_len = buffer.bytes;
        let start = Instant::now();
        if let Err(error) = insert_batch(&client, &config, metrics_family, key.table, &buffer).await
        {
            let (buffered_rows, buffered_bytes, active_buffers) =
                reinsert_failed_buffer(Arc::clone(&state), key, buffer, &error);
            record_failed_flush(
                metrics_family,
                rows_len,
                bytes_len,
                buffered_rows,
                buffered_bytes,
                active_buffers,
            );
            return Err(error);
        }

        let (buffered_rows, buffered_bytes, active_buffers) = {
            let mut state = state.lock().expect("ClickHouse writer state poisoned");
            state.buffered_rows = state.buffered_rows.saturating_sub(rows_len);
            state.buffered_bytes = state.buffered_bytes.saturating_sub(bytes_len);
            state.background_error = None;
            (
                state.buffered_rows,
                state.buffered_bytes,
                state.buffers.len(),
            )
        };

        record_successful_flush(
            metrics_family,
            rows_len,
            bytes_len,
            buffered_rows,
            buffered_bytes,
            active_buffers,
            start.elapsed(),
        );
        Ok(ClickHouseFlushOutcome {
            rows: rows_len,
            buffered_rows,
        })
    }

    async fn ensure_capacity(&self, row_bytes: usize) -> CarbonResult<()> {
        if self.has_capacity(row_bytes) {
            return Ok(());
        }

        Self::flush_stale_buffers(
            Arc::clone(&self.state),
            self.client.clone(),
            self.config.clone(),
            self.metrics_family,
        )
        .await?;

        if self.has_capacity(row_bytes) {
            return Ok(());
        }

        let keys = self.largest_buffer_keys();
        for key in keys {
            Self::flush_buffer_key(
                Arc::clone(&self.state),
                self.client.clone(),
                self.config.clone(),
                self.metrics_family,
                key,
            )
            .await?;

            if self.has_capacity(row_bytes) {
                return Ok(());
            }
        }

        record_backpressure_rejected(self.metrics_family);
        Err(Error::Custom(format!(
            "ClickHouse backpressure limit exceeded: row_bytes={row_bytes}, buffered_rows={}, buffered_bytes={}",
            self.buffered_rows(),
            self.buffered_bytes()
        )))
    }

    fn has_capacity(&self, row_bytes: usize) -> bool {
        let state = self.state.lock().expect("ClickHouse writer state poisoned");
        let batch = &self.config.batch_settings;

        let rows_ok = batch
            .max_buffered_rows
            .map(|max| state.buffered_rows.saturating_add(1) <= max)
            .unwrap_or(true);
        let bytes_ok = batch
            .max_buffered_bytes
            .map(|max| state.buffered_bytes.saturating_add(row_bytes) <= max)
            .unwrap_or(true);

        rows_ok && bytes_ok
    }

    fn largest_buffer_keys(&self) -> Vec<ClickHouseBufferKey> {
        let state = self.state.lock().expect("ClickHouse writer state poisoned");
        let mut entries = state
            .buffers
            .iter()
            .map(|(key, buffer)| (key.clone(), buffer.rows.len(), buffer.bytes))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| right.2.cmp(&left.2).then_with(|| right.1.cmp(&left.1)));
        entries.into_iter().map(|(key, _, _)| key).collect()
    }

    fn buffer_should_flush(&self, key: &ClickHouseBufferKey) -> bool {
        let state = self.state.lock().expect("ClickHouse writer state poisoned");
        let Some(buffer) = state.buffers.get(key) else {
            return false;
        };

        if buffer.rows.len() >= self.config.batch_settings.max_rows {
            return true;
        }

        self.config
            .batch_settings
            .max_bytes
            .map(|max_bytes| buffer.bytes >= max_bytes)
            .unwrap_or(false)
    }

    fn is_shutdown(state: &Arc<Mutex<ClickHouseWriterState>>) -> bool {
        state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .shutdown
    }

    fn take_background_error(&self) -> CarbonResult<()> {
        let error = self
            .state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .background_error
            .take();
        match error {
            Some(error) => Err(Error::Custom(error)),
            None => Ok(()),
        }
    }

    fn clear_background_error(&self) {
        self.state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .background_error = None;
    }
}

fn nonzero_interval(interval: Duration) -> Duration {
    if interval.is_zero() {
        Duration::from_millis(1)
    } else {
        interval
    }
}

fn reinsert_failed_buffer(
    state: Arc<Mutex<ClickHouseWriterState>>,
    key: ClickHouseBufferKey,
    mut buffer: ClickHouseBuffer,
    error: &Error,
) -> (usize, usize, usize) {
    let mut state = state.lock().expect("ClickHouse writer state poisoned");
    state.background_error = Some(error.to_string());
    match state.buffers.remove(&key) {
        Some(mut existing) => {
            buffer.rows.append(&mut existing.rows);
            buffer.bytes += existing.bytes;
            state.buffers.insert(
                key,
                ClickHouseBuffer {
                    rows: buffer.rows,
                    bytes: buffer.bytes,
                    last_flush: buffer.last_flush,
                },
            );
        }
        None => {
            state.buffers.insert(key, buffer);
        }
    }
    (
        state.buffered_rows,
        state.buffered_bytes,
        state.buffers.len(),
    )
}

async fn insert_batch(
    client: &reqwest::Client,
    config: &ClickHouseConfig,
    metrics_family: ClickHouseMetricsFamily,
    table: &str,
    buffer: &ClickHouseBuffer,
) -> CarbonResult<()> {
    let body = buffer_body(buffer);
    let query = format!("INSERT INTO {table} FORMAT JSONEachRow");

    let max_retries = config.retry_settings.max_retries;
    let mut attempt = 0usize;
    loop {
        let settings = batch_query_settings(config, table, &query, &body, attempt);
        match post_query_with_data(client, config, &query, &body, &settings).await {
            Ok(()) => return Ok(()),
            Err(error) if should_retry(&error, attempt, max_retries) => {
                record_retry(metrics_family);
                tokio::time::sleep(retry_delay(config, attempt)).await;
                attempt += 1;
            }
            Err(error) => return Err(Error::from(error)),
        }
    }
}

fn buffer_body(buffer: &ClickHouseBuffer) -> String {
    let mut body = String::with_capacity(buffer.bytes);
    for row in &buffer.rows {
        body.push_str(row);
        body.push('\n');
    }
    body
}

fn batch_query_settings(
    config: &ClickHouseConfig,
    table: &str,
    query: &str,
    body: &str,
    attempt: usize,
) -> Vec<ClickHouseQuerySetting> {
    let mut settings = config.insert_query_settings();
    settings.push(ClickHouseQuerySetting::new(
        "query_id",
        query_id(table, body, attempt),
    ));

    if config.deduplication_settings == ClickHouseDeduplicationSettings::ExactBatchHash {
        settings.push(ClickHouseQuerySetting::new(
            "insert_deduplication_token",
            deduplication_token(table, query, body),
        ));
    }

    settings
}

fn should_retry(error: &ClickHouseHttpError, attempt: usize, max_retries: usize) -> bool {
    error.kind.is_retryable() && attempt < max_retries
}

fn retry_delay(config: &ClickHouseConfig, attempt: usize) -> Duration {
    let base = config.retry_settings.initial_backoff;
    let multiplier = 1u32.checked_shl(attempt.min(16) as u32).unwrap_or(u32::MAX);
    let mut delay = base
        .saturating_mul(multiplier)
        .min(config.retry_settings.max_backoff);

    if config.retry_settings.jitter && !delay.is_zero() {
        let jitter_nanos = delay.as_nanos().saturating_div(10).min(u64::MAX as u128) as u64;
        if jitter_nanos > 0 {
            delay = delay.saturating_add(Duration::from_nanos(pseudo_jitter_nanos(jitter_nanos)));
        }
    }

    delay.min(config.retry_settings.max_backoff)
}

fn pseudo_jitter_nanos(max: u64) -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos() as u64 % max)
        .unwrap_or_default()
}

fn deduplication_token(table: &str, query: &str, body: &str) -> String {
    hash_hex([table, "\n", query, "\n", body])
}

fn query_id(table: &str, body: &str, attempt: usize) -> String {
    let attempt = attempt.to_string();
    format!(
        "carbon-clickhouse-{table}-{}",
        hash_hex([table, body, attempt.as_str()])
    )
}

fn hash_hex<'a>(parts: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part.as_bytes());
    }
    let digest = hasher.finalize();
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clickhouse::{
        metrics::register_clickhouse_metrics, rows::ClickHouseTable, ClickHouseAsyncInsertSettings,
        ClickHouseBatchSettings, ClickHouseDeduplicationSettings, ClickHouseHttpCompression,
        ClickHouseInsertSettings, ClickHouseRetrySettings, ClickHouseTransportSettings,
    };
    use crate::metrics::MetricsRegistry;
    use std::time::Duration;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    #[derive(Debug, Clone, serde::Serialize)]
    struct TestRow {
        id: u64,
        table: &'static str,
        partition: String,
    }

    impl ClickHouseTable for TestRow {
        fn table() -> &'static str {
            "test_table"
        }

        fn columns() -> Vec<&'static str> {
            vec!["id", "partition"]
        }

        fn create_table_sql(table_name: &str) -> String {
            format!(
                "CREATE TABLE IF NOT EXISTS {table_name} (id UInt64, partition String) ENGINE = MergeTree ORDER BY id"
            )
        }
    }

    impl ClickHouseRow for TestRow {
        fn table_name(&self) -> &'static str {
            self.table
        }

        fn partition_key(&self) -> String {
            self.partition.clone()
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
            "test_table".to_string(),
            "source".to_string(),
            "live".to_string(),
            "v1".to_string(),
            100,
            Duration::from_secs(60),
        )
    }

    fn config_with_max_rows(endpoint: String, max_rows: usize) -> ClickHouseConfig {
        ClickHouseConfig::new(
            endpoint,
            "default".to_string(),
            None,
            None,
            "test_table".to_string(),
            "source".to_string(),
            "live".to_string(),
            "v1".to_string(),
            max_rows,
            Duration::from_secs(60),
        )
    }

    fn config_with_flush_interval(endpoint: String, flush_interval: Duration) -> ClickHouseConfig {
        ClickHouseConfig::new(
            endpoint,
            "default".to_string(),
            None,
            None,
            "test_table".to_string(),
            "source".to_string(),
            "live".to_string(),
            "v1".to_string(),
            100,
            flush_interval,
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

    async fn start_clickhouse_server_with_responses(
        responses: Vec<&'static str>,
    ) -> (String, tokio::task::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();

            for response in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buffer = vec![0u8; 8192];
                let n = socket.read(&mut buffer).await.unwrap();
                requests.push(String::from_utf8_lossy(&buffer[..n]).to_string());
                socket.write_all(response.as_bytes()).await.unwrap();
            }

            requests
        });

        (endpoint, handle)
    }

    async fn start_clickhouse_binary_server(
        expected_requests: usize,
    ) -> (String, tokio::task::JoinHandle<Vec<Vec<u8>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();

            for _ in 0..expected_requests {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buffer = vec![0u8; 8192];
                let n = socket.read(&mut buffer).await.unwrap();
                requests.push(buffer[..n].to_vec());
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

    #[test]
    fn buffered_rows_are_tracked() {
        let writer = ClickHouseBatchWriter::<TestRow>::new(config());
        assert_eq!(writer.buffered_rows(), 0);
    }

    #[tokio::test]
    async fn flush_of_empty_writer_returns_zero_rows() {
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config());
        let outcome = writer.flush().await.unwrap();
        assert_eq!(outcome.rows, 0);
        assert_eq!(outcome.buffered_rows, 0);
        writer.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn buffers_are_partitioned_by_table_and_partition() {
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config());

        let first = writer
            .buffer_row(TestRow {
                id: 1,
                table: "first_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        let second = writer
            .buffer_row(TestRow {
                id: 2,
                table: "second_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(
            first,
            ClickHouseBufferOutcome::Buffered { buffered_rows: 1 }
        );
        assert_eq!(
            second,
            ClickHouseBufferOutcome::Buffered { buffered_rows: 2 }
        );
        assert_eq!(writer.buffered_rows(), 2);
        assert_eq!(writer.buffer_count(), 2);
    }

    #[tokio::test]
    async fn hot_buffer_size_flush_does_not_flush_cold_buffers() {
        let (endpoint, server) = start_clickhouse_server(1).await;
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config_with_max_rows(endpoint, 2));

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "hot_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        writer
            .buffer_row(TestRow {
                id: 2,
                table: "cold_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        let outcome = writer
            .buffer_row(TestRow {
                id: 3,
                table: "hot_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();

        let requests = server.await.unwrap().join("\n");
        assert_eq!(
            outcome,
            ClickHouseBufferOutcome::Flushed(ClickHouseFlushOutcome {
                rows: 2,
                buffered_rows: 1,
            })
        );
        assert_eq!(writer.buffered_rows(), 1);
        assert_eq!(writer.buffer_count(), 1);
        assert!(requests.contains("hot_table"));
        assert!(!requests.contains("cold_table"));
        assert!(requests.contains("\"id\":1"));
        assert!(requests.contains("\"id\":3"));
        writer.shutdown().await.unwrap_err();
    }

    #[tokio::test]
    async fn hot_buffer_byte_flush_does_not_flush_cold_buffers() {
        let (endpoint, server) = start_clickhouse_server(1).await;
        let config = config_with_endpoint(endpoint).with_batch_settings(ClickHouseBatchSettings {
            max_rows: 100,
            max_bytes: Some(80),
            max_buffered_rows: None,
            max_buffered_bytes: None,
            flush_interval: Duration::from_secs(60),
        });
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config);

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "hot_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        writer
            .buffer_row(TestRow {
                id: 2,
                table: "cold_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        let outcome = writer
            .buffer_row(TestRow {
                id: 3,
                table: "hot_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();

        let requests = server.await.unwrap().join("\n");
        assert_eq!(
            outcome,
            ClickHouseBufferOutcome::Flushed(ClickHouseFlushOutcome {
                rows: 2,
                buffered_rows: 1,
            })
        );
        assert_eq!(writer.buffered_rows(), 1);
        assert!(writer.buffered_bytes() > 0);
        assert!(requests.contains("hot_table"));
        assert!(!requests.contains("cold_table"));
        writer.shutdown().await.unwrap_err();
    }

    #[tokio::test]
    async fn background_timer_flushes_idle_stale_buffer() {
        let (endpoint, server) = start_clickhouse_server(1).await;
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config_with_flush_interval(
            endpoint,
            Duration::from_millis(20),
        ));

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "timer_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();

        let requests = server.await.unwrap().join("\n");
        assert!(requests.contains("timer_table"));
        assert!(requests.contains("\"id\":1"));

        tokio::time::sleep(Duration::from_millis(10)).await;
        assert_eq!(writer.buffered_rows(), 0);
        writer.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn flush_inserts_rows_grouped_by_table_and_partition() {
        let (endpoint, server) = start_clickhouse_server(2).await;
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config_with_endpoint(endpoint));

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "first_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        writer
            .buffer_row(TestRow {
                id: 2,
                table: "second_table",
                partition: "2027".to_string(),
            })
            .await
            .unwrap();

        let outcome = writer.flush().await.unwrap();
        let requests = server.await.unwrap().join("\n");

        assert_eq!(outcome.rows, 2);
        assert_eq!(outcome.buffered_rows, 0);
        assert!(requests.contains("first_table"));
        assert!(requests.contains("second_table"));
        assert!(requests.contains("\"partition\":\"2026\""));
        assert!(requests.contains("\"partition\":\"2027\""));
        writer.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn failed_buffer_flush_reinserts_only_that_buffer() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut buffer = vec![0u8; 8192];
            let n = socket.read(&mut buffer).await.unwrap();
            let request = String::from_utf8_lossy(&buffer[..n]).to_string();
            socket
                .write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 4\r\n\r\nfail")
                .await
                .unwrap();
            request
        });
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config_with_max_rows(endpoint, 2));

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "failing_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        writer
            .buffer_row(TestRow {
                id: 2,
                table: "cold_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        let error = writer
            .buffer_row(TestRow {
                id: 3,
                table: "failing_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap_err();

        let request = server.await.unwrap();
        assert!(error.to_string().contains("500 Internal Server Error"));
        assert_eq!(writer.buffered_rows(), 3);
        assert_eq!(writer.buffer_count(), 2);
        assert!(request.contains("failing_table"));
        assert!(!request.contains("cold_table"));
        writer.shutdown().await.unwrap_err();
    }

    #[tokio::test]
    async fn global_row_cap_drains_before_buffering_new_row() {
        let (endpoint, server) = start_clickhouse_server(2).await;
        let config = config_with_endpoint(endpoint).with_batch_settings(ClickHouseBatchSettings {
            max_rows: 100,
            max_bytes: None,
            max_buffered_rows: Some(1),
            max_buffered_bytes: None,
            flush_interval: Duration::from_secs(60),
        });
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config);

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "first_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        writer
            .buffer_row(TestRow {
                id: 2,
                table: "second_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(writer.buffered_rows(), 1);
        writer.shutdown().await.unwrap();
        let requests = server.await.unwrap().join("\n");
        assert!(requests.contains("first_table"));
        assert!(requests.contains("second_table"));
    }

    #[tokio::test]
    async fn global_byte_cap_rejects_row_that_cannot_fit_after_drain() {
        register_clickhouse_metrics();
        let before_rejected = counter_value("clickhouse.instructions.backpressure_rejected");
        let config = config().with_batch_settings(ClickHouseBatchSettings {
            max_rows: 100,
            max_bytes: None,
            max_buffered_rows: None,
            max_buffered_bytes: Some(1),
            flush_interval: Duration::from_secs(60),
        });
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config);

        let error = writer
            .buffer_row(TestRow {
                id: 1,
                table: "oversized_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("ClickHouse backpressure limit exceeded"));
        assert_eq!(writer.buffered_rows(), 0);
        assert_eq!(writer.buffered_bytes(), 0);
        assert!(counter_value("clickhouse.instructions.backpressure_rejected") > before_rejected);
    }

    #[tokio::test]
    async fn retryable_http_errors_are_retried() {
        register_clickhouse_metrics();
        let before_retries = counter_value("clickhouse.instructions.retries");
        let (endpoint, server) = start_clickhouse_server_with_responses(vec![
            "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 4\r\n\r\nfail",
            "HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n",
        ])
        .await;
        let config = config_with_endpoint(endpoint).with_retry_settings(ClickHouseRetrySettings {
            max_retries: 1,
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(1),
            jitter: false,
        });
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config);

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "retry_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        writer.flush().await.unwrap();

        let requests = server.await.unwrap();
        assert_eq!(requests.len(), 2);
        assert!(counter_value("clickhouse.instructions.retries") > before_retries);
        writer.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn permanent_http_errors_are_not_retried() {
        let (endpoint, server) = start_clickhouse_server_with_responses(vec![
            "HTTP/1.1 400 Bad Request\r\nContent-Length: 6\r\n\r\nschema",
        ])
        .await;
        let config = config_with_endpoint(endpoint).with_retry_settings(ClickHouseRetrySettings {
            max_retries: 3,
            initial_backoff: Duration::from_millis(1),
            max_backoff: Duration::from_millis(1),
            jitter: false,
        });
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config);

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "permanent_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        let error = writer.flush().await.unwrap_err();

        let requests = server.await.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(error.to_string().contains("400 Bad Request"));
        assert_eq!(writer.buffered_rows(), 1);
        writer.shutdown().await.unwrap_err();
    }

    #[tokio::test]
    async fn gzip_mode_sends_compressed_insert_body() {
        let (endpoint, server) = start_clickhouse_binary_server(1).await;
        let config =
            config_with_endpoint(endpoint).with_transport_settings(ClickHouseTransportSettings {
                compression: ClickHouseHttpCompression::Gzip,
                ..ClickHouseTransportSettings::default()
            });
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config);

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "gzip_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        writer.flush().await.unwrap();

        let request = server.await.unwrap().pop().unwrap();
        let request_text = String::from_utf8_lossy(&request);
        assert!(request.windows(2).any(|bytes| bytes == [0x1f, 0x8b]));
        assert!(request_text
            .to_ascii_lowercase()
            .contains("content-encoding: gzip"));
        writer.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn shutdown_drains_all_buffers() {
        let (endpoint, server) = start_clickhouse_server(2).await;
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config_with_endpoint(endpoint));

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "first_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        writer
            .buffer_row(TestRow {
                id: 2,
                table: "second_table",
                partition: "2027".to_string(),
            })
            .await
            .unwrap();

        let outcome = writer.shutdown().await.unwrap();
        let requests = server.await.unwrap().join("\n");

        assert_eq!(outcome.rows, 2);
        assert_eq!(outcome.buffered_rows, 0);
        assert!(requests.contains("first_table"));
        assert!(requests.contains("second_table"));
    }

    #[tokio::test]
    async fn async_insert_settings_are_sent_with_wait_enabled() {
        let (endpoint, server) = start_clickhouse_server(1).await;
        let config = config_with_endpoint(endpoint).with_insert_settings(
            ClickHouseInsertSettings::AsyncWait(ClickHouseAsyncInsertSettings {
                busy_timeout_ms: Some(250),
                max_data_size: Some(1_000_000),
                max_query_number: Some(8),
                deduplicate: Some(false),
            }),
        );
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config);

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "async_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();
        writer.flush().await.unwrap();

        let request = server.await.unwrap().join("\n");
        assert!(request.contains("async_insert=1"));
        assert!(request.contains("wait_for_async_insert=1"));
        assert!(!request.contains("wait_for_async_insert=0"));
        assert!(request.contains("async_insert_busy_timeout_ms=250"));
        assert!(request.contains("async_insert_max_data_size=1000000"));
        assert!(request.contains("async_insert_max_query_number=8"));
        assert!(request.contains("async_insert_deduplicate=0"));
        writer.shutdown().await.unwrap();
    }

    #[test]
    fn exact_batch_deduplication_token_is_stable_for_identical_batches() {
        let first = deduplication_token("table", "INSERT INTO table FORMAT JSONEachRow", "body\n");
        let second = deduplication_token("table", "INSERT INTO table FORMAT JSONEachRow", "body\n");
        let changed_body =
            deduplication_token("table", "INSERT INTO table FORMAT JSONEachRow", "changed\n");
        let changed_table =
            deduplication_token("other", "INSERT INTO other FORMAT JSONEachRow", "body\n");

        assert_eq!(first, second);
        assert_ne!(first, changed_body);
        assert_ne!(first, changed_table);
    }

    #[test]
    fn batch_query_settings_merge_insert_dedup_and_query_id_settings() {
        let config = config()
            .with_insert_settings(ClickHouseInsertSettings::AsyncWait(
                ClickHouseAsyncInsertSettings {
                    busy_timeout_ms: Some(250),
                    max_data_size: None,
                    max_query_number: None,
                    deduplicate: None,
                },
            ))
            .with_deduplication_settings(ClickHouseDeduplicationSettings::ExactBatchHash);

        let settings = batch_query_settings(
            &config,
            "settings_table",
            "INSERT INTO settings_table FORMAT JSONEachRow",
            "{\"id\":1}\n",
            2,
        );

        assert!(settings
            .iter()
            .any(|setting| setting.name == "date_time_input_format"));
        assert!(settings
            .iter()
            .any(|setting| setting.name == "async_insert" && setting.value == "1"));
        assert!(settings
            .iter()
            .any(|setting| setting.name == "wait_for_async_insert" && setting.value == "1"));
        assert!(settings
            .iter()
            .any(|setting| setting.name == "async_insert_busy_timeout_ms"));
        assert!(settings
            .iter()
            .any(|setting| setting.name == "insert_deduplication_token"));
        assert!(settings.iter().any(|setting| setting.name == "query_id"
            && setting
                .value
                .starts_with("carbon-clickhouse-settings_table-")));
    }

    #[test]
    fn retry_delay_with_jitter_stays_capped_by_max_backoff() {
        let config = config().with_retry_settings(ClickHouseRetrySettings {
            max_retries: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_millis(100),
            jitter: true,
        });

        assert!(retry_delay(&config, 8) <= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn writer_snapshot_and_metrics_track_bytes_and_buffers() {
        register_clickhouse_metrics();
        let before_inserted_bytes = counter_value("clickhouse.instructions.inserted_bytes");
        let (endpoint, server) = start_clickhouse_server(1).await;
        let mut writer = ClickHouseBatchWriter::<TestRow>::new(config_with_endpoint(endpoint));

        writer
            .buffer_row(TestRow {
                id: 1,
                table: "metrics_table",
                partition: "2026".to_string(),
            })
            .await
            .unwrap();

        let snapshot = writer.snapshot();
        assert_eq!(snapshot.buffered_rows, 1);
        assert!(snapshot.buffered_bytes > 0);
        assert_eq!(snapshot.active_buffers, 1);
        assert!(gauge_value("clickhouse.instructions.buffered_bytes") > 0.0);

        writer.flush().await.unwrap();
        let _ = server.await.unwrap();
        assert!(counter_value("clickhouse.instructions.inserted_bytes") > before_inserted_bytes);
        assert_eq!(writer.snapshot().buffered_bytes, 0);
        assert_eq!(writer.snapshot().active_buffers, 0);
        writer.shutdown().await.unwrap();
    }
}
