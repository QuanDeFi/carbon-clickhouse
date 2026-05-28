# Scene 01: Architecture And Tutorial Path

Carbon turns Solana RPC data into typed Rust records. The ClickHouse sink takes
those decoded records and writes them into generated landing tables.

In this walkthrough, we start the two canary examples as live runs. Then we
watch the running pipelines in Grafana, compare that with ClickHouse-side
activity in ClickStack, inspect the landing tables in ClickHouse Play, and
finish with the production boundaries.
