use crate::{
    clickhouse::{
        config::ClickHouseQuerySetting,
        http::{client_from_config, post_query, post_query_text},
        ClickHouseConfig,
    },
    error::{CarbonResult, Error},
};

pub trait ClickHouseSchema {
    fn operations(config: &ClickHouseConfig) -> Vec<String>;

    fn managed_tables(_config: &ClickHouseConfig) -> Vec<ClickHouseManagedTable> {
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseColumnDefinition {
    pub name: String,
    pub clickhouse_type: String,
    pub add_column_sql: String,
    pub modify_column_sql: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseManagedTable {
    pub name: String,
    pub create_table_sql: String,
    pub drop_table_sql: String,
    pub engine: Option<String>,
    pub partition_by: Option<String>,
    pub order_by: Option<String>,
    pub columns: Vec<ClickHouseColumnDefinition>,
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
        let managed_tables = S::managed_tables(&self.config);
        if managed_tables.is_empty() {
            return self.execute_queries(S::operations(&self.config)).await;
        }

        self.execute_managed_tables(&managed_tables).await
    }

    pub async fn execute_managed_tables(
        &self,
        tables: &[ClickHouseManagedTable],
    ) -> CarbonResult<()> {
        for table in tables {
            self.execute_query(&table.create_table_sql).await?;
            self.reconcile_table(table).await?;
        }

        Ok(())
    }

    async fn reconcile_table(&self, table: &ClickHouseManagedTable) -> CarbonResult<()> {
        self.validate_table_layout(table).await?;
        let live_columns = self.fetch_column_types(&table.name).await?;

        for expected in &table.columns {
            let Some(live_type) = live_columns.get(&expected.name) else {
                self.execute_query(&expected.add_column_sql).await?;
                continue;
            };

            if same_clickhouse_type(live_type, &expected.clickhouse_type) {
                continue;
            }

            if is_safe_enum_extension(live_type, &expected.clickhouse_type) {
                self.execute_query(&expected.modify_column_sql).await?;
            } else {
                return Err(schema_drift_error(
                    table,
                    expected,
                    live_type,
                    "type drift is not a safe enum extension",
                ));
            }
        }

        Ok(())
    }

    pub async fn fetch_column_types(
        &self,
        table_name: &str,
    ) -> CarbonResult<std::collections::HashMap<String, String>> {
        let query = format!(
            "SELECT name, type FROM system.columns WHERE database = currentDatabase() AND table = {} FORMAT TSVRaw",
            quote_sql_string(table_name)
        );
        let body = post_query_text(
            &self.client,
            &self.config,
            &query,
            &[ClickHouseQuerySetting::new(
                "date_time_input_format",
                "best_effort",
            )],
        )
        .await?;

        Ok(parse_columns_tsv(&body))
    }

    async fn validate_table_layout(&self, table: &ClickHouseManagedTable) -> CarbonResult<()> {
        if table.engine.is_none() && table.partition_by.is_none() && table.order_by.is_none() {
            return Ok(());
        }

        let query = format!(
            "SELECT engine, partition_key, sorting_key FROM system.tables WHERE database = currentDatabase() AND name = {} FORMAT TSVRaw",
            quote_sql_string(&table.name)
        );
        let body = post_query_text(
            &self.client,
            &self.config,
            &query,
            &[ClickHouseQuerySetting::new(
                "date_time_input_format",
                "best_effort",
            )],
        )
        .await?;
        let Some(layout) = parse_table_layout_tsv(&body) else {
            return Err(Error::Custom(format!(
                "ClickHouse schema drift for {}: table is missing after create",
                table.name
            )));
        };

        if let Some(expected) = &table.engine {
            if normalize_layout_fragment(&layout.engine) != normalize_layout_fragment(expected) {
                return Err(table_layout_drift_error(
                    table,
                    "engine",
                    &layout.engine,
                    expected,
                ));
            }
        }
        if let Some(expected) = &table.partition_by {
            if normalize_layout_fragment(&layout.partition_by)
                != normalize_layout_fragment(expected)
            {
                return Err(table_layout_drift_error(
                    table,
                    "partition key",
                    &layout.partition_by,
                    expected,
                ));
            }
        }
        if let Some(expected) = &table.order_by {
            if normalize_order_by(&layout.order_by) != normalize_order_by(expected) {
                return Err(table_layout_drift_error(
                    table,
                    "sorting key",
                    &layout.order_by,
                    expected,
                ));
            }
        }

        Ok(())
    }
}

fn schema_drift_error(
    table: &ClickHouseManagedTable,
    expected: &ClickHouseColumnDefinition,
    live_type: &str,
    reason: &str,
) -> Error {
    Error::Custom(format!(
        "ClickHouse schema drift for {}.{}: live type {}, expected type {} ({reason})",
        table.name, expected.name, live_type, expected.clickhouse_type
    ))
}

fn parse_columns_tsv(body: &str) -> std::collections::HashMap<String, String> {
    body.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '\t');
            let name = parts.next()?.to_string();
            let column_type = parts.next()?.to_string();
            Some((name, column_type))
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableLayout {
    engine: String,
    partition_by: String,
    order_by: String,
}

fn parse_table_layout_tsv(body: &str) -> Option<TableLayout> {
    let line = body.lines().next()?;
    let mut parts = line.splitn(3, '\t');
    Some(TableLayout {
        engine: parts.next()?.to_string(),
        partition_by: parts.next()?.to_string(),
        order_by: parts.next()?.to_string(),
    })
}

fn table_layout_drift_error(
    table: &ClickHouseManagedTable,
    field: &str,
    live: &str,
    expected: &str,
) -> Error {
    Error::Custom(format!(
        "ClickHouse schema drift for {} {field}: live {live}, expected {expected}",
        table.name
    ))
}

fn normalize_layout_fragment(value: &str) -> String {
    normalize_clickhouse_type(value)
}

fn normalize_order_by(value: &str) -> String {
    let normalized = normalize_layout_fragment(value);
    normalized
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .unwrap_or(&normalized)
        .to_string()
}

fn quote_sql_string(value: &str) -> String {
    format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn same_clickhouse_type(left: &str, right: &str) -> bool {
    normalize_clickhouse_type(left) == normalize_clickhouse_type(right)
}

fn is_safe_enum_extension(live_type: &str, expected_type: &str) -> bool {
    let live = normalize_clickhouse_type(live_type);
    let expected = normalize_clickhouse_type(expected_type);
    if live == expected {
        return true;
    }

    let Some(live_shape) = enum_placeholder_shape(&live) else {
        return false;
    };
    let Some(expected_shape) = enum_placeholder_shape(&expected) else {
        return false;
    };
    if live_shape.shape != expected_shape.shape
        || live_shape.enums.len() != expected_shape.enums.len()
        || live_shape.enums.is_empty()
    {
        return false;
    }

    live_shape
        .enums
        .iter()
        .zip(expected_shape.enums.iter())
        .all(|(live_enum, expected_enum)| enum_is_subset_with_same_values(live_enum, expected_enum))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnumShape {
    shape: String,
    enums: Vec<Vec<(String, i32)>>,
}

fn enum_placeholder_shape(input: &str) -> Option<EnumShape> {
    let mut output = String::with_capacity(input.len());
    let mut enums = Vec::new();
    let mut index = 0;

    while index < input.len() {
        if let Some(kind_len) = enum_prefix_len(&input[index..]) {
            let open = index + kind_len;
            let close = matching_paren(input, open)?;
            let body = &input[open + 1..close];
            enums.push(parse_enum_values(body)?);
            output.push_str("Enum()");
            index = close + 1;
        } else {
            let ch = input[index..].chars().next()?;
            output.push(ch);
            index += ch.len_utf8();
        }
    }

    Some(EnumShape {
        shape: output,
        enums,
    })
}

fn enum_prefix_len(input: &str) -> Option<usize> {
    if input.starts_with("Enum8(") {
        Some("Enum8".len())
    } else if input.starts_with("Enum16(") {
        Some("Enum16".len())
    } else if input.starts_with("Enum(") {
        Some("Enum".len())
    } else {
        None
    }
}

fn matching_paren(input: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in input[open..].char_indices() {
        let index = open + offset;
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '\'' => in_string = true,
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }

    None
}

fn parse_enum_values(body: &str) -> Option<Vec<(String, i32)>> {
    let mut values = Vec::new();
    for entry in split_top_level_commas(body) {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }

        let mut chars = entry.char_indices();
        if chars.next()?.1 != '\'' {
            return None;
        }

        let mut escaped = false;
        let mut value = String::new();
        let mut end_quote = None;
        for (index, ch) in chars {
            if escaped {
                value.push(ch);
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                end_quote = Some(index);
                break;
            } else {
                value.push(ch);
            }
        }
        let rest = entry[end_quote? + 1..].trim();
        let rest = rest.strip_prefix('=')?.trim();
        let numeric = rest.parse::<i32>().ok()?;
        values.push((value, numeric));
    }

    Some(values)
}

fn split_top_level_commas(input: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (index, ch) in input.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '\'' => in_string = true,
            ',' => {
                parts.push(&input[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(&input[start..]);
    parts
}

fn enum_is_subset_with_same_values(live: &[(String, i32)], expected: &[(String, i32)]) -> bool {
    let expected_by_name: std::collections::HashMap<&str, i32> = expected
        .iter()
        .map(|(name, value)| (name.as_str(), *value))
        .collect();

    live.iter()
        .all(|(name, value)| expected_by_name.get(name.as_str()) == Some(value))
        && expected.len() >= live.len()
}

fn normalize_clickhouse_type(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_string = false;
    let mut escaped = false;

    for ch in input.chars() {
        if in_string {
            output.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '\'' {
                in_string = false;
            }
            continue;
        }

        if ch == '\'' {
            in_string = true;
            output.push(ch);
        } else if !ch.is_whitespace() {
            output.push(ch);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::*;

    fn config(endpoint: String) -> ClickHouseConfig {
        ClickHouseConfig::new(
            endpoint,
            "default".to_string(),
            None,
            None,
            "test_table".to_string(),
            "source".to_string(),
            "live".to_string(),
            "v1".to_string(),
            100,
            Duration::from_secs(60),
        )
    }

    fn managed_test_table(expected_type: &str) -> ClickHouseManagedTable {
        ClickHouseManagedTable {
            name: "managed_landing".to_string(),
            create_table_sql: "CREATE TABLE IF NOT EXISTS managed_landing (route_plan Array(Tuple(swap Tuple(variant Enum16('A' = 0, 'B' = 1))))) ENGINE = MergeTree ORDER BY tuple()".to_string(),
            drop_table_sql: "DROP TABLE IF EXISTS managed_landing".to_string(),
            engine: Some("MergeTree".to_string()),
            partition_by: Some(String::new()),
            order_by: Some("tuple()".to_string()),
            columns: vec![ClickHouseColumnDefinition {
                name: "route_plan".to_string(),
                clickhouse_type: expected_type.to_string(),
                add_column_sql: format!(
                    "ALTER TABLE managed_landing ADD COLUMN IF NOT EXISTS route_plan {expected_type}"
                ),
                modify_column_sql: format!(
                    "ALTER TABLE managed_landing MODIFY COLUMN route_plan {expected_type}"
                ),
            }],
        }
    }

    async fn start_admin_server(
        responses: Vec<String>,
    ) -> (String, tokio::task::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();

            for response_body in responses {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut buffer = vec![0u8; 16 * 1024];
                let n = socket.read(&mut buffer).await.unwrap();
                requests.push(String::from_utf8_lossy(&buffer[..n]).to_string());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
                socket.write_all(response.as_bytes()).await.unwrap();
            }

            requests
        });

        (endpoint, handle)
    }

    fn request_body(request: &str) -> &str {
        request.split("\r\n\r\n").nth(1).unwrap_or_default()
    }

    #[test]
    fn parse_columns_tsv_reads_name_and_type() {
        let columns = parse_columns_tsv("a\tString\nroute_plan\tArray(Tuple(x Enum8('A' = 0)))\n");

        assert_eq!(columns.get("a"), Some(&"String".to_string()));
        assert_eq!(
            columns.get("route_plan"),
            Some(&"Array(Tuple(x Enum8('A' = 0)))".to_string())
        );
    }

    #[test]
    fn parse_table_layout_tsv_reads_engine_and_keys() {
        let layout = parse_table_layout_tsv(
            "MergeTree\ttoYear(partition_time)\tprogram_id, family_name, instruction_id, slot\n",
        )
        .unwrap();

        assert_eq!(layout.engine, "MergeTree");
        assert_eq!(layout.partition_by, "toYear(partition_time)");
        assert_eq!(
            normalize_order_by(&layout.order_by),
            normalize_order_by("(program_id, family_name, instruction_id, slot)")
        );
    }

    #[test]
    fn enum_extension_is_safe_for_nested_types() {
        let live = "Array(Tuple(swap Tuple(variant Enum16('A' = 0, 'B' = 1), amount UInt64)))";
        let expected =
            "Array(Tuple(swap Tuple(variant Enum16('A' = 0, 'B' = 1, 'C' = 2), amount UInt64)))";

        assert!(is_safe_enum_extension(live, expected));
    }

    #[test]
    fn enum8_to_enum16_extension_is_safe_when_values_are_stable() {
        let live = "Tuple(variant Enum8('A' = 0, 'B' = 1))";
        let expected = "Tuple(variant Enum16('A' = 0, 'B' = 1, 'C' = 2))";

        assert!(is_safe_enum_extension(live, expected));
    }

    #[test]
    fn changed_enum_numeric_value_is_unsafe() {
        let live = "Enum8('A' = 0, 'B' = 1)";
        let expected = "Enum8('A' = 0, 'B' = 2)";

        assert!(!is_safe_enum_extension(live, expected));
    }

    #[test]
    fn removed_enum_value_is_unsafe() {
        let live = "Enum8('A' = 0, 'B' = 1)";
        let expected = "Enum8('A' = 0)";

        assert!(!is_safe_enum_extension(live, expected));
    }

    #[test]
    fn tuple_shape_change_is_unsafe() {
        let live = "Tuple(variant Enum8('A' = 0), amount UInt64)";
        let expected = "Tuple(variant Enum8('A' = 0, 'B' = 1), amount UInt64, extra String)";

        assert!(!is_safe_enum_extension(live, expected));
    }

    #[tokio::test]
    async fn managed_schema_adds_missing_columns() {
        let expected = "Array(Tuple(swap Tuple(variant Enum16('A' = 0, 'B' = 1))))";
        let (endpoint, server) = start_admin_server(vec![
            String::new(),
            "MergeTree\t\ttuple()\n".to_string(),
            String::new(),
            String::new(),
        ])
        .await;
        let admin = ClickHouseAdmin::new(config(endpoint));

        admin
            .execute_managed_tables(&[managed_test_table(expected)])
            .await
            .unwrap();

        let requests = server.await.unwrap();
        assert_eq!(requests.len(), 4);
        assert!(request_body(&requests[0]).contains("CREATE TABLE IF NOT EXISTS managed_landing"));
        assert!(request_body(&requests[1]).contains("SELECT engine, partition_key, sorting_key"));
        assert!(request_body(&requests[2]).contains("SELECT name, type FROM system.columns"));
        assert!(request_body(&requests[3]).contains("ADD COLUMN IF NOT EXISTS route_plan"));
    }

    #[tokio::test]
    async fn managed_schema_modifies_safe_enum_extensions() {
        let live = "route_plan\tArray(Tuple(swap Tuple(variant Enum16('A' = 0))))\n";
        let expected = "Array(Tuple(swap Tuple(variant Enum16('A' = 0, 'B' = 1))))";
        let (endpoint, server) = start_admin_server(vec![
            String::new(),
            "MergeTree\t\ttuple()\n".to_string(),
            live.to_string(),
            String::new(),
        ])
        .await;
        let admin = ClickHouseAdmin::new(config(endpoint));

        admin
            .execute_managed_tables(&[managed_test_table(expected)])
            .await
            .unwrap();

        let requests = server.await.unwrap();
        assert_eq!(requests.len(), 4);
        assert!(request_body(&requests[3]).contains("MODIFY COLUMN route_plan"));
    }

    #[tokio::test]
    async fn managed_schema_rejects_unsafe_type_drift() {
        let live = "route_plan\tString\n";
        let expected = "Array(Tuple(swap Tuple(variant Enum16('A' = 0, 'B' = 1))))";
        let (endpoint, server) = start_admin_server(vec![
            String::new(),
            "MergeTree\t\ttuple()\n".to_string(),
            live.to_string(),
        ])
        .await;
        let admin = ClickHouseAdmin::new(config(endpoint));

        let error = admin
            .execute_managed_tables(&[managed_test_table(expected)])
            .await
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("ClickHouse schema drift for managed_landing.route_plan"));
        let requests = server.await.unwrap();
        assert_eq!(requests.len(), 3);
    }

    #[tokio::test]
    async fn managed_schema_rejects_layout_drift() {
        let expected = "Array(Tuple(swap Tuple(variant Enum16('A' = 0, 'B' = 1))))";
        let (endpoint, server) = start_admin_server(vec![
            String::new(),
            "MergeTree\tpartition_slot\ttuple()\n".to_string(),
        ])
        .await;
        let admin = ClickHouseAdmin::new(config(endpoint));

        let error = admin
            .execute_managed_tables(&[managed_test_table(expected)])
            .await
            .unwrap_err();

        assert!(error
            .to_string()
            .contains("ClickHouse schema drift for managed_landing partition key"));
        let requests = server.await.unwrap();
        assert_eq!(requests.len(), 2);
    }
}
