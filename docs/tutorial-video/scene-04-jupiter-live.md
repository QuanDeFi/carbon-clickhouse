Now Jupiter starts with plain `cargo run`.

The env selects live mode and async-wait inserts. ClickHouse groups concurrent small writes; Carbon waits for acknowledgement.

Startup reconciles generated Jupiter tables, connects to RPC, decodes live activity, and keeps writing rows.
