use crate::error::CarbonResult;
use sha2::{Digest, Sha256};

pub trait ClickHouseTable {
    fn table() -> &'static str;
    fn columns() -> Vec<&'static str>;
    fn create_table_sql(table_name: &str) -> String;
}

pub trait ClickHouseRow:
    serde::Serialize + Clone + Send + Sync + std::fmt::Debug + 'static
{
    fn partition_key(&self) -> String;

    fn to_json_line(&self) -> CarbonResult<String> {
        serde_json::to_string(self).map_err(|e| crate::error::Error::Custom(e.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClickHouseRowContext {
    pub source_name: String,
    pub mode: String,
    pub decoder_version: String,
}

pub trait ClickHouseRows<R: ClickHouseRow>: Send + Sync + 'static {
    fn clickhouse_rows(&self, context: &ClickHouseRowContext) -> Vec<R>;
}

pub fn deterministic_event_id(
    program_id: &[u8],
    signature: &str,
    absolute_path: &[u8],
    event_type: &str,
    event_seq: u32,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(program_id);
    hasher.update(signature.as_bytes());
    hasher.update(absolute_path);
    hasher.update(event_type.as_bytes());
    hasher.update(event_seq.to_le_bytes());
    let digest = hasher.finalize();
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::deterministic_event_id;

    #[test]
    fn deterministic_event_id_is_stable() {
        let first = deterministic_event_id(b"program", "sig", &[1, 2, 3], "swap_event", 0);
        let second = deterministic_event_id(b"program", "sig", &[1, 2, 3], "swap_event", 0);
        assert_eq!(first, second);
    }
}
