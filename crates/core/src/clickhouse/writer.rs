use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio_util::sync::CancellationToken;

use crate::{
    clickhouse::{
        config::ClickHouseConfig,
        http::post_query_with_data,
        metrics::{
            record_buffered_rows, record_failed_flush, record_successful_flush,
            ClickHouseMetricsFamily,
        },
        rows::ClickHouseRow,
    },
    error::{CarbonResult, Error},
};

pub struct ClickHouseBatchWriter<R: ClickHouseRow> {
    config: ClickHouseConfig,
    client: reqwest::Client,
    state: Arc<Mutex<ClickHouseWriterState<R>>>,
    shutdown_token: CancellationToken,
    background_task: Option<tokio::task::JoinHandle<()>>,
    metrics_family: ClickHouseMetricsFamily,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ClickHouseBufferKey {
    table: &'static str,
    partition: String,
}

#[derive(Debug)]
struct ClickHouseBuffer<R> {
    rows: Vec<R>,
    last_flush: Instant,
}

#[derive(Debug)]
struct ClickHouseWriterState<R> {
    buffers: HashMap<ClickHouseBufferKey, ClickHouseBuffer<R>>,
    buffered_rows: usize,
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

impl<R: ClickHouseRow> ClickHouseBatchWriter<R> {
    pub fn new(config: ClickHouseConfig) -> Self {
        Self::new_with_metrics(config, ClickHouseMetricsFamily::Instructions)
    }

    pub(crate) fn new_with_metrics(
        config: ClickHouseConfig,
        metrics_family: ClickHouseMetricsFamily,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
            state: Arc::new(Mutex::new(ClickHouseWriterState {
                buffers: HashMap::new(),
                buffered_rows: 0,
                background_error: None,
                shutdown: false,
            })),
            shutdown_token: CancellationToken::new(),
            background_task: None,
            metrics_family,
        }
    }

    pub fn buffered_rows(&self) -> usize {
        self.state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .buffered_rows
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

        let key = ClickHouseBufferKey {
            table: row.table_name(),
            partition: row.partition_key(),
        };
        let buffered_rows = {
            let mut state = self.state.lock().expect("ClickHouse writer state poisoned");
            if state.shutdown {
                return Err(Error::Custom(
                    "Cannot buffer rows after ClickHouse writer shutdown".to_string(),
                ));
            }

            state
                .buffers
                .entry(key.clone())
                .or_insert_with(|| ClickHouseBuffer {
                    rows: Vec::new(),
                    last_flush: Instant::now(),
                })
                .rows
                .push(row);
            state.buffered_rows += 1;
            state.buffered_rows
        };

        record_buffered_rows(self.metrics_family, buffered_rows);

        if self.buffer_len(&key) >= self.config.max_rows {
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
        let interval = nonzero_interval(config.flush_interval);

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
        state: Arc<Mutex<ClickHouseWriterState<R>>>,
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
        state: Arc<Mutex<ClickHouseWriterState<R>>>,
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
                    if now.duration_since(buffer.last_flush) >= config.flush_interval {
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
        state: Arc<Mutex<ClickHouseWriterState<R>>>,
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
        state: Arc<Mutex<ClickHouseWriterState<R>>>,
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
        let start = Instant::now();
        if let Err(error) = insert_batch(&client, &config, key.table, &buffer.rows).await {
            let buffered_rows = reinsert_failed_buffer(Arc::clone(&state), key, buffer, &error);
            record_failed_flush(metrics_family, rows_len, buffered_rows);
            return Err(error);
        }

        let buffered_rows = {
            let mut state = state.lock().expect("ClickHouse writer state poisoned");
            state.buffered_rows -= rows_len;
            state.background_error = None;
            state.buffered_rows
        };

        record_successful_flush(metrics_family, rows_len, buffered_rows, start.elapsed());
        Ok(ClickHouseFlushOutcome {
            rows: rows_len,
            buffered_rows,
        })
    }

    fn buffer_len(&self, key: &ClickHouseBufferKey) -> usize {
        self.state
            .lock()
            .expect("ClickHouse writer state poisoned")
            .buffers
            .get(key)
            .map(|buffer| buffer.rows.len())
            .unwrap_or_default()
    }

    fn is_shutdown(state: &Arc<Mutex<ClickHouseWriterState<R>>>) -> bool {
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

fn reinsert_failed_buffer<R: ClickHouseRow>(
    state: Arc<Mutex<ClickHouseWriterState<R>>>,
    key: ClickHouseBufferKey,
    mut buffer: ClickHouseBuffer<R>,
    error: &Error,
) -> usize {
    let mut state = state.lock().expect("ClickHouse writer state poisoned");
    state.background_error = Some(error.to_string());
    match state.buffers.remove(&key) {
        Some(mut existing) => {
            buffer.rows.append(&mut existing.rows);
            state.buffers.insert(
                key,
                ClickHouseBuffer {
                    rows: buffer.rows,
                    last_flush: buffer.last_flush,
                },
            );
        }
        None => {
            state.buffers.insert(key, buffer);
        }
    }
    state.buffered_rows
}

async fn insert_batch<R: ClickHouseRow>(
    client: &reqwest::Client,
    config: &ClickHouseConfig,
    table: &str,
    rows: &[R],
) -> CarbonResult<()> {
    let mut body = String::new();
    for row in rows {
        body.push_str(&row.to_json_line()?);
        body.push('\n');
    }

    let settings = config.insert_query_settings();
    post_query_with_data(
        client,
        config,
        &format!("INSERT INTO {table} FORMAT JSONEachRow"),
        &body,
        &settings,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clickhouse::{
        rows::ClickHouseTable, ClickHouseAsyncInsertSettings, ClickHouseInsertSettings,
    };
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
}
