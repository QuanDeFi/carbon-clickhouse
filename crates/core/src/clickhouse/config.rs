use std::time::Duration;

use crate::{
    clickhouse::rows::ClickHouseRowContext,
    error::{CarbonResult, Error},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ClickHouseQuerySetting {
    pub name: &'static str,
    pub value: String,
}

impl ClickHouseQuerySetting {
    pub fn new(name: &'static str, value: impl Into<String>) -> Self {
        Self {
            name,
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClickHouseInsertSettings {
    Sync,
    AsyncWait(ClickHouseAsyncInsertSettings),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClickHouseAsyncInsertSettings {
    pub busy_timeout_ms: Option<u64>,
    pub max_data_size: Option<u64>,
    pub max_query_number: Option<u64>,
    pub deduplicate: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseBatchSettings {
    pub max_rows: usize,
    pub max_bytes: Option<usize>,
    pub max_buffered_rows: Option<usize>,
    pub max_buffered_bytes: Option<usize>,
    pub flush_interval: Duration,
}

impl ClickHouseBatchSettings {
    pub fn new(max_rows: usize, flush_interval: Duration) -> Self {
        Self {
            max_rows,
            max_bytes: None,
            max_buffered_rows: None,
            max_buffered_bytes: None,
            flush_interval,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClickHouseHttpCompression {
    #[default]
    None,
    Gzip,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClickHouseTransportSettings {
    pub request_timeout: Option<Duration>,
    pub connect_timeout: Option<Duration>,
    pub pool_idle_timeout: Option<Duration>,
    pub pool_max_idle_per_host: Option<usize>,
    pub compression: ClickHouseHttpCompression,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseRetrySettings {
    pub max_retries: usize,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub jitter: bool,
}

impl Default for ClickHouseRetrySettings {
    fn default() -> Self {
        Self {
            max_retries: 0,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
            jitter: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClickHouseDeduplicationSettings {
    #[default]
    Disabled,
    ExactBatchHash,
}

#[derive(Debug, Clone)]
pub struct ClickHouseConfig {
    pub endpoint: String,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub table: String,
    pub source_name: String,
    pub mode: String,
    pub decoder_version: String,
    pub max_rows: usize,
    pub flush_interval: Duration,
    pub insert_settings: ClickHouseInsertSettings,
    pub batch_settings: ClickHouseBatchSettings,
    pub transport_settings: ClickHouseTransportSettings,
    pub retry_settings: ClickHouseRetrySettings,
    pub deduplication_settings: ClickHouseDeduplicationSettings,
}

impl ClickHouseConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        endpoint: String,
        database: String,
        username: Option<String>,
        password: Option<String>,
        table: String,
        source_name: String,
        mode: String,
        decoder_version: String,
        max_rows: usize,
        flush_interval: Duration,
    ) -> Self {
        Self {
            endpoint,
            database,
            username,
            password,
            table,
            source_name,
            mode,
            decoder_version,
            max_rows,
            flush_interval,
            insert_settings: ClickHouseInsertSettings::Sync,
            batch_settings: ClickHouseBatchSettings::new(max_rows, flush_interval),
            transport_settings: ClickHouseTransportSettings::default(),
            retry_settings: ClickHouseRetrySettings::default(),
            deduplication_settings: ClickHouseDeduplicationSettings::default(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_database_url(
        database_url: &str,
        database: String,
        table: String,
        source_name: String,
        mode: String,
        decoder_version: String,
        max_rows: usize,
        flush_interval: Duration,
    ) -> CarbonResult<Self> {
        let parsed = url::Url::parse(database_url).map_err(|err| {
            Error::Custom(format!("Invalid DATABASE_URL={database_url:?}: {err}"))
        })?;

        let scheme = parsed.scheme();
        let host = parsed
            .host_str()
            .ok_or_else(|| Error::Custom("DATABASE_URL must include a host".to_string()))?;
        let endpoint = match parsed.port() {
            Some(port) => format!("{scheme}://{host}:{port}"),
            None => format!("{scheme}://{host}"),
        };

        let username = if parsed.username().is_empty() {
            None
        } else {
            Some(parsed.username().to_string())
        };
        let password = parsed.password().map(|value| value.to_string());

        Ok(Self::new(
            endpoint,
            database,
            username,
            password,
            table,
            source_name,
            mode,
            decoder_version,
            max_rows,
            flush_interval,
        ))
    }

    pub fn row_context(&self) -> ClickHouseRowContext {
        ClickHouseRowContext {
            source_name: self.source_name.clone(),
            mode: self.mode.clone(),
            decoder_version: self.decoder_version.clone(),
        }
    }

    pub fn with_insert_settings(mut self, settings: ClickHouseInsertSettings) -> Self {
        self.insert_settings = settings;
        self
    }

    pub fn with_batch_settings(mut self, settings: ClickHouseBatchSettings) -> Self {
        self.max_rows = settings.max_rows;
        self.flush_interval = settings.flush_interval;
        self.batch_settings = settings;
        self
    }

    pub fn with_transport_settings(mut self, settings: ClickHouseTransportSettings) -> Self {
        self.transport_settings = settings;
        self
    }

    pub fn with_retry_settings(mut self, settings: ClickHouseRetrySettings) -> Self {
        self.retry_settings = settings;
        self
    }

    pub fn with_deduplication_settings(
        mut self,
        settings: ClickHouseDeduplicationSettings,
    ) -> Self {
        self.deduplication_settings = settings;
        self
    }

    pub(crate) fn insert_query_settings(&self) -> Vec<ClickHouseQuerySetting> {
        let mut settings = vec![ClickHouseQuerySetting::new(
            "date_time_input_format",
            "best_effort",
        )];

        match &self.insert_settings {
            ClickHouseInsertSettings::Sync => {}
            ClickHouseInsertSettings::AsyncWait(async_settings) => {
                settings.push(ClickHouseQuerySetting::new("async_insert", "1"));
                settings.push(ClickHouseQuerySetting::new("wait_for_async_insert", "1"));

                if let Some(value) = async_settings.busy_timeout_ms {
                    settings.push(ClickHouseQuerySetting::new(
                        "async_insert_busy_timeout_ms",
                        value.to_string(),
                    ));
                }
                if let Some(value) = async_settings.max_data_size {
                    settings.push(ClickHouseQuerySetting::new(
                        "async_insert_max_data_size",
                        value.to_string(),
                    ));
                }
                if let Some(value) = async_settings.max_query_number {
                    settings.push(ClickHouseQuerySetting::new(
                        "async_insert_max_query_number",
                        value.to_string(),
                    ));
                }
                if let Some(value) = async_settings.deduplicate {
                    settings.push(ClickHouseQuerySetting::new(
                        "async_insert_deduplicate",
                        if value { "1" } else { "0" },
                    ));
                }
            }
        }

        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ClickHouseConfig {
        ClickHouseConfig::new(
            "http://localhost:8123".to_string(),
            "default".to_string(),
            None,
            None,
            "table".to_string(),
            "source".to_string(),
            "mode".to_string(),
            "v1".to_string(),
            100,
            Duration::from_secs(1),
        )
    }

    #[test]
    fn insert_query_settings_are_sync_by_default() {
        let settings = config().insert_query_settings();

        assert_eq!(settings.len(), 1);
        assert_eq!(settings[0].name, "date_time_input_format");
        assert_eq!(settings[0].value, "best_effort");
    }

    #[test]
    fn legacy_constructors_populate_production_defaults() {
        let config = config();

        assert_eq!(config.batch_settings.max_rows, config.max_rows);
        assert_eq!(config.batch_settings.flush_interval, config.flush_interval);
        assert_eq!(config.batch_settings.max_bytes, None);
        assert_eq!(config.batch_settings.max_buffered_rows, None);
        assert_eq!(config.batch_settings.max_buffered_bytes, None);
        assert_eq!(
            config.transport_settings,
            ClickHouseTransportSettings::default()
        );
        assert_eq!(config.retry_settings, ClickHouseRetrySettings::default());
        assert_eq!(
            config.deduplication_settings,
            ClickHouseDeduplicationSettings::Disabled
        );
    }

    #[test]
    fn batch_settings_builder_updates_legacy_fields() {
        let config = config().with_batch_settings(ClickHouseBatchSettings {
            max_rows: 10,
            max_bytes: Some(1024),
            max_buffered_rows: Some(100),
            max_buffered_bytes: Some(4096),
            flush_interval: Duration::from_secs(2),
        });

        assert_eq!(config.max_rows, 10);
        assert_eq!(config.flush_interval, Duration::from_secs(2));
        assert_eq!(config.batch_settings.max_bytes, Some(1024));
        assert_eq!(config.batch_settings.max_buffered_rows, Some(100));
        assert_eq!(config.batch_settings.max_buffered_bytes, Some(4096));
    }

    #[test]
    fn async_insert_settings_always_wait_for_clickhouse() {
        let settings = config()
            .with_insert_settings(ClickHouseInsertSettings::AsyncWait(
                ClickHouseAsyncInsertSettings {
                    busy_timeout_ms: Some(250),
                    max_data_size: Some(1_000_000),
                    max_query_number: Some(8),
                    deduplicate: Some(true),
                },
            ))
            .insert_query_settings();

        assert!(settings
            .iter()
            .any(|setting| setting.name == "async_insert" && setting.value == "1"));
        assert!(settings
            .iter()
            .any(|setting| setting.name == "wait_for_async_insert" && setting.value == "1"));
        assert!(!settings
            .iter()
            .any(|setting| { setting.name == "wait_for_async_insert" && setting.value == "0" }));
        assert!(settings.iter().any(|setting| {
            setting.name == "async_insert_busy_timeout_ms" && setting.value == "250"
        }));
        assert!(settings.iter().any(|setting| {
            setting.name == "async_insert_max_data_size" && setting.value == "1000000"
        }));
        assert!(settings.iter().any(|setting| {
            setting.name == "async_insert_max_query_number" && setting.value == "8"
        }));
        assert!(settings
            .iter()
            .any(|setting| { setting.name == "async_insert_deduplicate" && setting.value == "1" }));
    }
}
