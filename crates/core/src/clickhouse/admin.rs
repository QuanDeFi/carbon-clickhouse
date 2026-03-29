use reqwest::{header::CONTENT_LENGTH, StatusCode};

use crate::{
    clickhouse::ClickHouseConfig,
    error::{CarbonResult, Error},
};

pub trait ClickHouseSchema {
    fn operations(config: &ClickHouseConfig) -> Vec<String>;
}

pub struct ClickHouseAdmin {
    client: reqwest::Client,
    config: ClickHouseConfig,
}

impl ClickHouseAdmin {
    pub fn new(config: ClickHouseConfig) -> Self {
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

    pub async fn execute_query(&self, query: &str) -> CarbonResult<()> {
        let request = self
            .client
            .post(&self.config.endpoint)
            .query(&[
                ("database", self.config.database.as_str()),
                ("query", query),
                ("date_time_input_format", "best_effort"),
            ])
            .header(CONTENT_LENGTH, 0)
            .body(String::new());

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

    pub async fn execute_queries<I>(&self, queries: I) -> CarbonResult<()>
    where
        I: IntoIterator<Item = String>,
    {
        for query in queries {
            self.execute_query(&query).await?;
        }

        Ok(())
    }

    pub async fn execute_schema<S: ClickHouseSchema>(&self) -> CarbonResult<()> {
        self.execute_queries(S::operations(&self.config)).await
    }
}
