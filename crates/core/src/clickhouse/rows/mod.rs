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
    fn table_name(&self) -> &'static str;
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
    hex_digest(hasher.finalize())
}

pub fn deterministic_instruction_id(
    program_id: &[u8],
    signature: &str,
    absolute_path: &[u8],
    instruction_type: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(program_id);
    hasher.update(signature.as_bytes());
    hasher.update(absolute_path);
    hasher.update(instruction_type.as_bytes());
    hex_digest(hasher.finalize())
}

pub fn deterministic_account_id(
    program_id: &[u8],
    pubkey: &[u8],
    slot: u64,
    account_type: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(program_id);
    hasher.update(pubkey);
    hasher.update(slot.to_le_bytes());
    hasher.update(account_type.as_bytes());
    hex_digest(hasher.finalize())
}

pub fn deterministic_transaction_id(scope: &str, signature: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(scope.as_bytes());
    hasher.update(signature.as_bytes());
    hex_digest(hasher.finalize())
}

pub fn deterministic_account_deletion_id(
    pubkey: &[u8],
    slot: u64,
    transaction_signature: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pubkey);
    hasher.update(slot.to_le_bytes());
    if let Some(signature) = transaction_signature {
        hasher.update(signature.as_bytes());
    }
    hex_digest(hasher.finalize())
}

pub fn deterministic_block_details_id(slot: u64, block_hash: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(slot.to_le_bytes());
    if let Some(block_hash) = block_hash {
        hasher.update(block_hash.as_bytes());
    }
    hex_digest(hasher.finalize())
}

fn hex_digest(digest: impl IntoIterator<Item = u8>) -> String {
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::{
        deterministic_account_deletion_id, deterministic_account_id,
        deterministic_block_details_id, deterministic_event_id, deterministic_instruction_id,
        deterministic_transaction_id,
    };

    #[test]
    fn deterministic_event_id_is_stable() {
        let first = deterministic_event_id(b"program", "sig", &[1, 2, 3], "swap_event", 0);
        let second = deterministic_event_id(b"program", "sig", &[1, 2, 3], "swap_event", 0);
        assert_eq!(first, second);
    }

    #[test]
    fn deterministic_instruction_id_is_stable() {
        let first = deterministic_instruction_id(b"program", "sig", &[1, 2, 3], "swap");
        let second = deterministic_instruction_id(b"program", "sig", &[1, 2, 3], "swap");
        assert_eq!(first, second);
    }

    #[test]
    fn deterministic_account_id_is_stable() {
        let first = deterministic_account_id(b"program", b"pubkey", 42, "mint");
        let second = deterministic_account_id(b"program", b"pubkey", 42, "mint");
        assert_eq!(first, second);
    }

    #[test]
    fn deterministic_transaction_id_is_stable_and_scoped() {
        let first = deterministic_transaction_id("generic", "sig");
        let second = deterministic_transaction_id("generic", "sig");
        let other_scope = deterministic_transaction_id("jupiter_swap", "sig");

        assert_eq!(first, second);
        assert_ne!(first, other_scope);
    }

    #[test]
    fn deterministic_account_deletion_id_is_stable_and_signature_sensitive() {
        let first = deterministic_account_deletion_id(b"pubkey", 42, Some("sig"));
        let second = deterministic_account_deletion_id(b"pubkey", 42, Some("sig"));
        let without_signature = deterministic_account_deletion_id(b"pubkey", 42, None);

        assert_eq!(first, second);
        assert_ne!(first, without_signature);
    }

    #[test]
    fn deterministic_block_details_id_is_stable_and_hash_sensitive() {
        let first = deterministic_block_details_id(42, Some("hash"));
        let second = deterministic_block_details_id(42, Some("hash"));
        let without_hash = deterministic_block_details_id(42, None);

        assert_eq!(first, second);
        assert_ne!(first, without_hash);
    }
}
