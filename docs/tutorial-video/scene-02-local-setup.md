Next we show the example environment files, not private values.

They define the database URL, RPC URL, metrics ports, log level, Jupiter slot
range, and optional async-wait inserts. Leaving the Jupiter slot range empty
makes it follow live head.

The full ClickHouse sink config lives in `crates/core/src/clickhouse/config.rs`.
