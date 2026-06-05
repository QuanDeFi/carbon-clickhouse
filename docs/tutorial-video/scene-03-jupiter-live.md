Now we start the Jupiter Swap example with plain `cargo run`.

The env selects live mode and async-wait inserts. Startup first builds the
ClickHouse configs and reconciles the generated Jupiter instruction, event, and
TokenLedger account landing tables.

Because the slot range is empty, the example follows the most recent slot and
also attaches the TokenLedger account fetcher. The block crawler connects
to RPC, the Jupiter decoder emits typed activity, and the ClickHouse processors
keep writing rows to the database.
