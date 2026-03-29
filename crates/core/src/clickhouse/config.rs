use std::time::Duration;

use crate::error::{CarbonResult, Error};

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
        let parsed = url::Url::parse(database_url)
            .map_err(|err| Error::Custom(format!("Invalid DATABASE_URL={database_url:?}: {err}")))?;

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
}
