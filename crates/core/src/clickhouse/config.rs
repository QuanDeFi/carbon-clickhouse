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
