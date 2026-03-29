use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    time::Instant,
};

use reqwest::{header::CONTENT_LENGTH, StatusCode};

use crate::{
    clickhouse::{config::ClickHouseConfig, rows::ClickHouseRow},
    error::{CarbonResult, Error},
};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

trait ClickHouseTransport: Send + Sync {
    fn insert_json_each_row<'a>(
        &'a self,
        table: &'a str,
        body: &'a str,
    ) -> BoxFuture<'a, CarbonResult<()>>;
}

#[derive(Clone)]
struct HttpClickHouseTransport {
    client: reqwest::Client,
    config: ClickHouseConfig,
}

impl HttpClickHouseTransport {
    fn new(config: ClickHouseConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    fn apply_auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if let Some(username) = &self.config.username {
            request.basic_auth(username, self.config.password.as_ref())
        } else {
            request
        }
    }

    async fn send_post_query(&self, query: &str, body: String) -> CarbonResult<()> {
        let content_length = body.len();
        let request = self
            .client
            .post(&self.config.endpoint)
            .query(&[
                ("database", self.config.database.as_str()),
                ("query", query),
                ("date_time_input_format", "best_effort"),
            ])
            .header(CONTENT_LENGTH, content_length)
            .body(body);

        let response = self
            .apply_auth(request)
            .send()
            .await
            .map_err(|e| Error::Custom(format!("ClickHouse request failed: {e}")))?;

        if response.status() == StatusCode::OK {
            return Ok(());
        }

        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read response body>".to_string());
        Err(Error::Custom(format!(
            "ClickHouse request failed with status {status}: {body}"
        )))
    }
}

impl ClickHouseTransport for HttpClickHouseTransport {
    fn insert_json_each_row<'a>(
        &'a self,
        table: &'a str,
        body: &'a str,
    ) -> BoxFuture<'a, CarbonResult<()>> {
        let query = format!("INSERT INTO {table} FORMAT JSONEachRow");
        Box::pin(async move { self.send_post_query(&query, body.to_string()).await })
    }
}

pub struct ClickHouseBatchWriter<R: ClickHouseRow> {
    config: ClickHouseConfig,
    transport: Box<dyn ClickHouseTransport>,
    buffers: HashMap<String, Vec<R>>,
    last_flush: Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickHouseBufferOutcome {
    Buffered,
    Flushed { rows: usize },
}

impl<R: ClickHouseRow> ClickHouseBatchWriter<R> {
    pub fn new(config: ClickHouseConfig) -> Self {
        Self::new_with_transport(
            config.clone(),
            Box::new(HttpClickHouseTransport::new(config)),
        )
    }

    fn new_with_transport(
        config: ClickHouseConfig,
        transport: Box<dyn ClickHouseTransport>,
    ) -> Self {
        Self {
            config,
            transport,
            buffers: HashMap::new(),
            last_flush: Instant::now(),
        }
    }

    pub fn buffered_rows(&self) -> usize {
        self.buffers.values().map(Vec::len).sum()
    }

    pub async fn buffer_row(&mut self, row: R) -> CarbonResult<ClickHouseBufferOutcome> {
        let partition_key = row.partition_key();
        self.buffers.entry(partition_key).or_default().push(row);

        if self.should_flush() {
            let rows = self.flush().await?;
            return Ok(ClickHouseBufferOutcome::Flushed { rows });
        }

        Ok(ClickHouseBufferOutcome::Buffered)
    }

    pub async fn flush(&mut self) -> CarbonResult<usize> {
        if self.buffered_rows() == 0 {
            self.last_flush = Instant::now();
            return Ok(0);
        }

        let mut flushed = 0usize;
        let keys: Vec<String> = self.buffers.keys().cloned().collect();
        for key in keys {
            let rows = self
                .buffers
                .get(&key)
                .cloned()
                .ok_or_else(|| Error::Custom(format!("Missing buffer for partition {key}")))?;

            self.insert_batch(&rows).await?;
            flushed += rows.len();
            self.buffers.remove(&key);
        }

        self.last_flush = Instant::now();
        Ok(flushed)
    }

    fn should_flush(&self) -> bool {
        self.buffered_rows() >= self.config.max_rows
            || self.last_flush.elapsed() >= self.config.flush_interval
    }

    async fn insert_batch(&self, rows: &[R]) -> CarbonResult<()> {
        let mut body = String::new();
        for row in rows {
            body.push_str(&row.to_json_line()?);
            body.push('\n');
        }

        match self
            .transport
            .insert_json_each_row(&self.config.table, &body)
            .await
        {
            Ok(()) => Ok(()),
            Err(_) => self
                .transport
                .insert_json_each_row(&self.config.table, &body)
                .await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clickhouse::rows::ClickHouseTable;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[derive(Debug, Clone, serde::Serialize)]
    struct TestRow {
        id: u64,
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
        fn partition_key(&self) -> String {
            self.partition.clone()
        }
    }

    #[derive(Default)]
    struct MockTransportState {
        inserts: Vec<String>,
        fail_first_insert: bool,
        insert_calls: usize,
    }

    #[derive(Clone, Default)]
    struct MockTransport {
        state: Arc<Mutex<MockTransportState>>,
    }

    impl ClickHouseTransport for MockTransport {
        fn insert_json_each_row<'a>(
            &'a self,
            _table: &'a str,
            body: &'a str,
        ) -> BoxFuture<'a, CarbonResult<()>> {
            let state = self.state.clone();
            let body = body.to_string();
            Box::pin(async move {
                let mut guard = state.lock().unwrap();
                guard.insert_calls += 1;
                if guard.fail_first_insert && guard.insert_calls == 1 {
                    return Err(Error::Custom("simulated insert failure".to_string()));
                }
                guard.inserts.push(body);
                Ok(())
            })
        }
    }

    fn config() -> ClickHouseConfig {
        ClickHouseConfig::new(
            "http://localhost:8123".to_string(),
            "default".to_string(),
            None,
            None,
            "test_table".to_string(),
            "source".to_string(),
            "live".to_string(),
            "v1".to_string(),
            2,
            Duration::from_millis(10),
        )
    }

    #[tokio::test]
    async fn flush_on_row_threshold_succeeds() {
        let transport = MockTransport::default();
        let state = transport.state.clone();
        let mut writer = ClickHouseBatchWriter::new_with_transport(config(), Box::new(transport));

        assert_eq!(
            writer
                .buffer_row(TestRow {
                    id: 1,
                    partition: "2024".to_string(),
                })
                .await
                .unwrap(),
            ClickHouseBufferOutcome::Buffered
        );

        assert_eq!(
            writer
                .buffer_row(TestRow {
                    id: 2,
                    partition: "2024".to_string(),
                })
                .await
                .unwrap(),
            ClickHouseBufferOutcome::Flushed { rows: 2 }
        );

        let guard = state.lock().unwrap();
        assert_eq!(guard.inserts.len(), 1);
        assert!(guard.inserts[0].contains("\"id\":1"));
        assert!(guard.inserts[0].contains("\"id\":2"));
    }

    #[tokio::test]
    async fn flush_on_timer_succeeds() {
        let transport = MockTransport::default();
        let state = transport.state.clone();
        let mut writer = ClickHouseBatchWriter::new_with_transport(config(), Box::new(transport));

        writer
            .buffer_row(TestRow {
                id: 1,
                partition: "2024".to_string(),
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(15)).await;
        assert_eq!(
            writer
                .buffer_row(TestRow {
                    id: 2,
                    partition: "2024".to_string(),
                })
                .await
                .unwrap(),
            ClickHouseBufferOutcome::Flushed { rows: 2 }
        );

        let guard = state.lock().unwrap();
        assert_eq!(guard.inserts.len(), 1);
    }

    #[tokio::test]
    async fn graceful_shutdown_flushes_buffered_rows() {
        let transport = MockTransport::default();
        let state = transport.state.clone();
        let mut writer = ClickHouseBatchWriter::new_with_transport(config(), Box::new(transport));

        writer
            .buffer_row(TestRow {
                id: 1,
                partition: "2024".to_string(),
            })
            .await
            .unwrap();

        let flushed = writer.flush().await.unwrap();
        assert_eq!(flushed, 1);

        let guard = state.lock().unwrap();
        assert_eq!(guard.inserts.len(), 1);
        assert!(guard.inserts[0].contains("\"id\":1"));
    }

    #[tokio::test]
    async fn failed_batch_retry_preserves_exact_row_order() {
        let transport = MockTransport::default();
        transport.state.lock().unwrap().fail_first_insert = true;
        let state = transport.state.clone();
        let mut writer = ClickHouseBatchWriter::new_with_transport(config(), Box::new(transport));

        writer
            .buffer_row(TestRow {
                id: 1,
                partition: "2024".to_string(),
            })
            .await
            .unwrap();
        writer
            .buffer_row(TestRow {
                id: 2,
                partition: "2024".to_string(),
            })
            .await
            .unwrap();

        let guard = state.lock().unwrap();
        assert_eq!(guard.insert_calls, 2);
        assert_eq!(guard.inserts.len(), 1);
        assert_eq!(
            guard.inserts[0],
            "{\"id\":1,\"partition\":\"2024\"}\n{\"id\":2,\"partition\":\"2024\"}\n"
        );
    }
}
