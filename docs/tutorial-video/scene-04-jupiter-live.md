Now Jupiter starts from a plain `cargo run` command.

Its environment file already selects live mode and async-wait inserts. Startup
creates or reconciles generated Jupiter landing tables, connects to RPC, decodes
live Jupiter instructions and events, and keeps writing rows into ClickHouse
while we continue.
