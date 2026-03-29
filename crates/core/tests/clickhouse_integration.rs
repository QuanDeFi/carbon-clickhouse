#![cfg(feature = "clickhouse")]

use std::time::Duration;

use carbon_core::clickhouse::{
    rows::ClickHouseRow, ClickHouseAdmin, ClickHouseBatchWriter, ClickHouseBufferOutcome,
    ClickHouseConfig,
};
use carbon_core::error::{CarbonResult, Error};

#[derive(Debug, Clone, serde::Serialize)]
struct IntegrationRow {
    id: u64,
    partition_time: String,
    payload: String,
}

impl ClickHouseRow for IntegrationRow {
    fn partition_key(&self) -> String {
        self.partition_time[..4].to_string()
    }

    fn create_table_sql(table_name: &str) -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (\
                id UInt64,\
                partition_time DateTime64(3, 'UTC'),\
                payload String\
            ) ENGINE = MergeTree \
            PARTITION BY toYear(partition_time) \
            ORDER BY id"
        )
    }
}

fn integration_config(table: String, max_rows: usize, flush_interval: Duration) -> Option<ClickHouseConfig> {
    let endpoint = std::env::var("CLICKHOUSE_INTEGRATION_ENDPOINT").ok()?;
    let database = std::env::var("CLICKHOUSE_INTEGRATION_DATABASE")
        .unwrap_or_else(|_| "default".to_string());
    let username = std::env::var("CLICKHOUSE_INTEGRATION_USERNAME").ok();
    let password = std::env::var("CLICKHOUSE_INTEGRATION_PASSWORD").ok();

    Some(ClickHouseConfig::new(
        endpoint,
        database,
        username,
        password,
        table,
        "integration-test".to_string(),
        "test".to_string(),
        "v1".to_string(),
        max_rows,
        flush_interval,
    ))
}

async fn execute_query(config: &ClickHouseConfig, query: &str) -> CarbonResult<String> {
    let client = reqwest::Client::new();
    let mut request = client
        .post(&config.endpoint)
        .query(&[("database", config.database.as_str()), ("query", query)])
        .header(reqwest::header::CONTENT_LENGTH, 0)
        .body(String::new());

    if let Some(username) = &config.username {
        request = request.basic_auth(username, config.password.as_ref());
    }

    let response = request
        .send()
        .await
        .map_err(|e| Error::Custom(format!("ClickHouse integration request failed: {e}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "<failed to read response body>".to_string());
        return Err(Error::Custom(format!(
            "ClickHouse integration request failed with status {status}: {body}"
        )));
    }

    response
        .text()
        .await
        .map_err(|e| Error::Custom(format!("ClickHouse integration body read failed: {e}")))
}

async fn count_rows(config: &ClickHouseConfig) -> CarbonResult<u64> {
    let query = format!("SELECT count() FROM {}", config.table);
    let response = execute_query(config, &query).await?;
    response
        .trim()
        .parse::<u64>()
        .map_err(|e| Error::Custom(format!("Failed to parse ClickHouse count response: {e}")))
}

#[tokio::test]
async fn inserting_a_small_batch_succeeds_when_clickhouse_env_is_present() -> CarbonResult<()> {
    let table = format!("clickhouse_sink_test_{}", uuid::Uuid::new_v4().simple());
    let Some(config) = integration_config(table, 10, Duration::from_secs(60)) else {
        return Ok(());
    };

    let mut writer = ClickHouseBatchWriter::new(config.clone());
    ClickHouseAdmin::new(config.clone())
        .execute_query(&IntegrationRow::create_table_sql(&config.table))
        .await?;
    writer
        .buffer_row(IntegrationRow {
            id: 1,
            partition_time: "2024-01-01 00:00:00.000".to_string(),
            payload: "first".to_string(),
        })
        .await?;
    writer
        .buffer_row(IntegrationRow {
            id: 2,
            partition_time: "2024-01-01 00:00:00.000".to_string(),
            payload: "second".to_string(),
        })
        .await?;

    let flushed = writer.flush().await?;
    assert_eq!(flushed, 2);
    assert_eq!(count_rows(&config).await?, 2);

    Ok(())
}

#[tokio::test]
async fn row_threshold_flush_succeeds_when_clickhouse_env_is_present() -> CarbonResult<()> {
    let table = format!("clickhouse_sink_test_{}", uuid::Uuid::new_v4().simple());
    let Some(config) = integration_config(table, 2, Duration::from_secs(60)) else {
        return Ok(());
    };

    let mut writer = ClickHouseBatchWriter::new(config.clone());
    ClickHouseAdmin::new(config.clone())
        .execute_query(&IntegrationRow::create_table_sql(&config.table))
        .await?;
    let flushed = writer
        .buffer_row(IntegrationRow {
            id: 1,
            partition_time: "2024-01-01 00:00:00.000".to_string(),
            payload: "first".to_string(),
        })
        .await?;
    assert_eq!(flushed, ClickHouseBufferOutcome::Buffered);

    let flushed = writer
        .buffer_row(IntegrationRow {
            id: 2,
            partition_time: "2024-01-01 00:00:00.000".to_string(),
            payload: "second".to_string(),
        })
        .await?;
    assert_eq!(flushed, ClickHouseBufferOutcome::Flushed { rows: 2 });
    assert_eq!(count_rows(&config).await?, 2);

    Ok(())
}
