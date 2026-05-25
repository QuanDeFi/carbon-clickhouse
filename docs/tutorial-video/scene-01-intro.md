# Scene 01: Intro And Architecture

Carbon decodes Solana data into typed Rust structures. The ClickHouse sink
writes those decoded records into generated landing tables.

In this tutorial, we will run the Jupiter Swap and Token Program canaries,
validate rows in ClickHouse, and inspect the same pipeline through Prometheus
and Grafana.
