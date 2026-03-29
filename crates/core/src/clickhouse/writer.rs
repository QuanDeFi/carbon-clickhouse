use std::{collections::HashMap, time::Instant};

use crate::{
    clickhouse::{config::ClickHouseConfig, http::post_query_with_data, rows::ClickHouseRow},
    error::{CarbonResult, Error},
};

pub struct ClickHouseBatchWriter<R: ClickHouseRow> {
    config: ClickHouseConfig,
    client: reqwest::Client,
    buffers: HashMap<String, Vec<R>>,
    buffered_rows: usize,
    last_flush: Instant,
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
        let partition_key = row.partition_key();
        self.buffers.entry(partition_key).or_default().push(row);
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
        let keys: Vec<String> = self.buffers.keys().cloned().collect();
        for key in keys {
            let rows = self
                .buffers
                .remove(&key)
                .ok_or_else(|| Error::Custom(format!("Missing buffer for partition {key}")))?;

            if let Err(error) = self.insert_batch(&rows).await {
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

    async fn insert_batch(&self, rows: &[R]) -> CarbonResult<()> {
        let mut body = String::new();
        for row in rows {
            body.push_str(&row.to_json_line()?);
            body.push('\n');
        }

        post_query_with_data(
            &self.client,
            &self.config,
            &format!("INSERT INTO {} FORMAT JSONEachRow", self.config.table),
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
}
