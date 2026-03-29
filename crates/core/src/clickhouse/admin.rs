use crate::{
    clickhouse::{http::post_query, ClickHouseConfig},
    error::CarbonResult,
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

    pub async fn execute_query(&self, query: &str) -> CarbonResult<()> {
        post_query(
            &self.client,
            &self.config,
            query,
            &[("date_time_input_format", "best_effort")],
        )
        .await
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
