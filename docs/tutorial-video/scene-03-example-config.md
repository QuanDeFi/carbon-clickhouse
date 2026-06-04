The full ClickHouse sink runtime configuration lives in
crates/core/src/clickhouse/config.rs.

That file defines what can be configured: insert mode, async insert tuning,
batch sizing, transport behavior, retry policy, deduplication, database name,
table name, source name, mode, and decoder version.

The examples only need a small surface area: database URL, RPC URL, metrics
port, slot range where the example supports one, and the async-wait toggle.
Showing the environment files is enough for running the tutorial; this config
file is the reference when someone wants to see the full sink options.

The important default is simple: Carbon uses synchronous inserts unless the
example opts into async-wait inserts. In async-wait mode, ClickHouse can batch
the small writes server-side, and Carbon still waits for confirmation that the
data was flushed.
