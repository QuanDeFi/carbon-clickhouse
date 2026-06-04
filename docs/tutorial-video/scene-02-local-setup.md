Next we check where the example env values live.

The env files set database URL, RPC URL, metrics ports, log level, Jupiter slot range, and async-wait inserts. Empty Jupiter slots mean live mode.

For code wiring, read each `main.rs`. The full sink config is in `crates/core/src/clickhouse/config.rs`.
