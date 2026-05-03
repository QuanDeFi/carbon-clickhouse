use std::{collections::HashMap, time::Instant};

use crate::{
    clickhouse::{config::ClickHouseConfig, http::post_query_with_data, rows::ClickHouseRow},
    error::{CarbonResult, Error},
};

pub struct ClickHouseBatchWriter<R: ClickHouseRow> {
    config: ClickHouseConfig,
    client: reqwest::Client,
    buffers: HashMap<ClickHouseBufferKey, Vec<R>>,
    buffered_rows: usize,
    last_flush: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ClickHouseBufferKey {
    table: &'static str,
    partition: String,
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
        Self {
            client: reqwest::Client::new(),
            config,
            buffers: HashMap::new(),
            buffered_rows: 0,
            last_flush: Instant::now(),
        }
    }

    pub fn buffered_rows(&self) -> usize {
        self.buffered_rows
    }

    pub async fn buffer_row(&mut self, row: R) -> CarbonResult<ClickHouseBufferOutcome> {
        let key = ClickHouseBufferKey {
            table: row.table_name(),
            partition: row.partition_key(),
        };
        self.buffers.entry(key).or_default().push(row);
        self.buffered_rows += 1;

        if self.should_flush() {
            return self.flush().await.map(ClickHouseBufferOutcome::Flushed);
        }

        Ok(ClickHouseBufferOutcome::Buffered {
            buffered_rows: self.buffered_rows(),
        })
    }

    pub async fn flush(&mut self) -> CarbonResult<ClickHouseFlushOutcome> {
        if self.buffered_rows() == 0 {
            self.last_flush = Instant::now();
            return Ok(ClickHouseFlushOutcome {
                rows: 0,
                buffered_rows: 0,
            });
        }

        let mut flushed = 0usize;
        let keys: Vec<ClickHouseBufferKey> = self.buffers.keys().cloned().collect();
        for key in keys {
            let rows = self.buffers.remove(&key).ok_or_else(|| {
                Error::Custom(format!(
                    "Missing buffer for table {} partition {}",
                    key.table, key.partition
                ))
            })?;

            if let Err(error) = self.insert_batch(key.table, &rows).await {
                self.buffers.insert(key, rows);
                return Err(error);
            }

            self.buffered_rows -= rows.len();
            flushed += rows.len();
        }

        self.last_flush = Instant::now();
        Ok(ClickHouseFlushOutcome {
            rows: flushed,
            buffered_rows: self.buffered_rows,
        })
    }

    fn should_flush(&self) -> bool {
        self.buffered_rows() >= self.config.max_rows
            || self.last_flush.elapsed() >= self.config.flush_interval
    }

    async fn insert_batch(&self, table: &str, rows: &[R]) -> CarbonResult<()> {
        let mut body = String::new();
        for row in rows {
            body.push_str(&row.to_json_line()?);
            body.push('\n');
        }

        post_query_with_data(
            &self.client,
            &self.config,
            &format!("INSERT INTO {table} FORMAT JSONEachRow"),
            &body,
            &[("date_time_input_format", "best_effort")],
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clickhouse::rows::ClickHouseTable;
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
        assert_eq!(writer.buffers.len(), 2);
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
    }
}
