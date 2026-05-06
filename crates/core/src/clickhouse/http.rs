use std::{fmt, io::Write};

use flate2::{write::GzEncoder, Compression};
use reqwest::{header, StatusCode};

use crate::{
    clickhouse::{
        config::{ClickHouseHttpCompression, ClickHouseQuerySetting},
        ClickHouseConfig,
    },
    error::Error,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ClickHouseHttpErrorKind {
    Network,
    Timeout,
    RetryableStatus,
    PermanentStatus,
}

impl ClickHouseHttpErrorKind {
    pub(crate) fn is_retryable(self) -> bool {
        matches!(self, Self::Network | Self::Timeout | Self::RetryableStatus)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ClickHouseHttpError {
    pub kind: ClickHouseHttpErrorKind,
    message: String,
}

impl ClickHouseHttpError {
    fn request(error: reqwest::Error) -> Self {
        let kind = if error.is_timeout() {
            ClickHouseHttpErrorKind::Timeout
        } else {
            ClickHouseHttpErrorKind::Network
        };

        Self {
            kind,
            message: format!("ClickHouse request failed: {error}"),
        }
    }

    fn response(status: StatusCode, body: String) -> Self {
        let kind = if is_retryable_status(status, &body) {
            ClickHouseHttpErrorKind::RetryableStatus
        } else {
            ClickHouseHttpErrorKind::PermanentStatus
        };

        Self {
            kind,
            message: format!("ClickHouse request failed with status {status}: {body}"),
        }
    }
}

impl fmt::Display for ClickHouseHttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<ClickHouseHttpError> for Error {
    fn from(value: ClickHouseHttpError) -> Self {
        Error::Custom(value.to_string())
    }
}

pub(crate) fn client_from_config(config: &ClickHouseConfig) -> reqwest::Client {
    let mut builder = reqwest::Client::builder();

    if let Some(timeout) = config.transport_settings.request_timeout {
        builder = builder.timeout(timeout);
    }
    if let Some(timeout) = config.transport_settings.connect_timeout {
        builder = builder.connect_timeout(timeout);
    }
    if let Some(timeout) = config.transport_settings.pool_idle_timeout {
        builder = builder.pool_idle_timeout(timeout);
    }
    if let Some(max_idle) = config.transport_settings.pool_max_idle_per_host {
        builder = builder.pool_max_idle_per_host(max_idle);
    }
    if let Some(user_agent) = &config.transport_settings.user_agent {
        if let Ok(value) = header::HeaderValue::from_str(user_agent) {
            let mut headers = header::HeaderMap::new();
            headers.insert(header::USER_AGENT, value);
            builder = builder.default_headers(headers);
        }
    }

    builder.build().unwrap_or_else(|_| reqwest::Client::new())
}

fn apply_auth(
    config: &ClickHouseConfig,
    request: reqwest::RequestBuilder,
) -> reqwest::RequestBuilder {
    if let Some(username) = &config.username {
        request.basic_auth(username, config.password.as_ref())
    } else {
        request
    }
}

pub(crate) async fn post_query(
    client: &reqwest::Client,
    config: &ClickHouseConfig,
    query: &str,
    settings: &[ClickHouseQuerySetting],
) -> Result<(), ClickHouseHttpError> {
    let request = client
        .post(&config.endpoint)
        .query(&[("database", config.database.as_str())]);
    let request = settings.iter().fold(request, |request, setting| {
        request.query(&[(setting.name, setting.value.as_str())])
    });
    let response = apply_auth(config, request.body(query.to_owned()))
        .send()
        .await
        .map_err(ClickHouseHttpError::request)?;

    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "<failed to read response body>".to_string());
    Err(ClickHouseHttpError::response(status, body))
}

pub(crate) async fn post_query_with_data(
    client: &reqwest::Client,
    config: &ClickHouseConfig,
    query: &str,
    data: &str,
    settings: &[ClickHouseQuerySetting],
) -> Result<(), ClickHouseHttpError> {
    let request = client
        .post(&config.endpoint)
        .query(&[("database", config.database.as_str()), ("query", query)]);
    let request = settings.iter().fold(request, |request, setting| {
        request.query(&[(setting.name, setting.value.as_str())])
    });

    let request =
        match config.transport_settings.compression {
            ClickHouseHttpCompression::None => request.body(data.to_owned()),
            ClickHouseHttpCompression::Gzip => request
                .header(header::CONTENT_ENCODING, "gzip")
                .body(gzip(data).map_err(|message| ClickHouseHttpError {
                    kind: ClickHouseHttpErrorKind::PermanentStatus,
                    message,
                })?),
        };

    let response = apply_auth(config, request)
        .send()
        .await
        .map_err(ClickHouseHttpError::request)?;

    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    let body = response
        .text()
        .await
        .unwrap_or_else(|_| "<failed to read response body>".to_string());
    Err(ClickHouseHttpError::response(status, body))
}

fn gzip(data: &str) -> Result<Vec<u8>, String> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(data.as_bytes())
        .map_err(|error| format!("Failed to gzip ClickHouse request body: {error}"))?;
    encoder
        .finish()
        .map_err(|error| format!("Failed to finish gzip ClickHouse request body: {error}"))
}

fn is_retryable_status(status: StatusCode, body: &str) -> bool {
    if status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
    {
        return true;
    }

    let body = body.to_ascii_lowercase();
    body.contains("too many parts") || body.contains("too many inactive parts")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_timeout_status_is_retryable() {
        let error = ClickHouseHttpError::response(StatusCode::REQUEST_TIMEOUT, String::new());

        assert_eq!(error.kind, ClickHouseHttpErrorKind::RetryableStatus);
        assert!(error.kind.is_retryable());
    }

    #[test]
    fn too_many_parts_body_is_retryable_even_on_bad_request() {
        let error =
            ClickHouseHttpError::response(StatusCode::BAD_REQUEST, "Too many parts".to_string());

        assert_eq!(error.kind, ClickHouseHttpErrorKind::RetryableStatus);
        assert!(error.kind.is_retryable());
    }

    #[test]
    fn schema_bad_request_is_permanent() {
        let error =
            ClickHouseHttpError::response(StatusCode::BAD_REQUEST, "Unknown column".to_string());

        assert_eq!(error.kind, ClickHouseHttpErrorKind::PermanentStatus);
        assert!(!error.kind.is_retryable());
    }
}
