Next we show where the example configuration lives.

The env files set the database URL, RPC URL, metrics ports, log level, Jupiter
slot range, and the optional async-wait insert switch. Empty Jupiter start and
end slots mean pure live mode.

Sync inserts are the default. In this tutorial we opt into async-wait inserts
because both examples write small batches into the same ClickHouse server.
ClickHouse can group those writes, and Carbon still waits for acknowledgement.

For code wiring, read each `main.rs`. The full sink config surface is in
`crates/core/src/clickhouse/config.rs`.
