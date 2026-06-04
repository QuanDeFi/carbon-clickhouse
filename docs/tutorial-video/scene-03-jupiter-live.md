Now Jupiter starts with plain `cargo run`.

The env selects live mode and async-wait inserts. Startup first builds the
ClickHouse configs and reconciles the generated Jupiter instruction, event, and
TokenLedger account landing tables.

Because the slot range is empty, the example follows the finalized head and
also attaches the live TokenLedger account fetcher. The block crawler connects
to RPC, the Jupiter decoder emits typed activity, and the ClickHouse processors
keep writing rows while we continue.
