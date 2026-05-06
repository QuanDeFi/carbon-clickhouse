use crate::{
    clickhouse::{
        config::ClickHouseQuerySetting,
        http::{client_from_config, post_query},
        ClickHouseConfig,
    },
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
            client: client_from_config(&config),
            config,
        }
    }

    pub async fn execute_query(&self, query: &str) -> CarbonResult<()> {
        post_query(
            &self.client,
            &self.config,
            query,
            &[ClickHouseQuerySetting::new(
                "date_time_input_format",
                "best_effort",
            )],
        )
        .await?;
        Ok(())
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
