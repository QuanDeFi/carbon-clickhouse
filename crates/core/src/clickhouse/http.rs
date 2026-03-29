use crate::{
    clickhouse::ClickHouseConfig,
    error::{CarbonResult, Error},
};

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
    settings: &[(&str, &str)],
) -> CarbonResult<()> {
    let request = client
        .post(&config.endpoint)
        .query(&[("database", config.database.as_str())]);
    let request = settings.iter().fold(request, |request, (key, value)| {
        request.query(&[(*key, *value)])
    });
    let response = apply_auth(config, request.body(query.to_owned()))
        .send()
        .await
        .map_err(|e| Error::Custom(format!("ClickHouse request failed: {e}")))?;

    if response.status().is_success() {
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

pub(crate) async fn post_query_with_data(
    client: &reqwest::Client,
    config: &ClickHouseConfig,
    query: &str,
    data: &str,
    settings: &[(&str, &str)],
) -> CarbonResult<()> {
    let request = client
        .post(&config.endpoint)
        .query(&[("database", config.database.as_str()), ("query", query)]);
    let request = settings.iter().fold(request, |request, (key, value)| {
        request.query(&[(*key, *value)])
    });
    let response = apply_auth(config, request.body(data.to_owned()))
        .send()
        .await
        .map_err(|e| Error::Custom(format!("ClickHouse request failed: {e}")))?;

    if response.status().is_success() {
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
