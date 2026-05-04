# Production ClickHouse Design Notes

## Recommendation

Use one generated typed landing table per row family as the production default. A row family is one generated instruction row, one generated CPI-event row, or one generated account row. Do not collapse all decoders into one universal landing table as the main analytical path.

- Use per-family tables such as `jupiter_swap_route_instruction_landing`, `jupiter_swap_swap_event_landing`, and `token_program_mint_account_landing`.
- Use one table per decoder only when every emitted row in that decoder genuinely shares one stable schema and query pattern.
- Use a global raw/envelope table only as an optional audit or replay diagnostic layer, not as the typed landing table.

Typed per-family tables preserve ClickHouse columnar benefits, isolate schema evolution, and avoid a sparse universal table or JSON blob model.

## Carbon And Multi-Writer Topology

Carbon can register multiple datasources and multiple decoder/processor pipes in one `Pipeline`, but a pipeline processes its update channel through one event loop. Production should still assume multiple Carbon processes or multiple pipeline tasks writing to ClickHouse concurrently.

The sink is designed for that topology:

- each Carbon process owns local buffers,
- buffers are keyed by `(table, partition)`,
- size and timer flushes operate per buffer,
- `finalize()` drains every remaining buffer before shutdown.

## Insert Mode

Synchronous inserts are the default and should remain the default for backfills and deterministic ingestion.

Async inserts are an explicit production option for live deployments with many Carbon writers and small per-writer batches. Configure them through `ClickHouseConfig::with_insert_settings(ClickHouseInsertSettings::AsyncWait(...))`.

The sink intentionally does not expose `wait_for_async_insert=0`. Async mode always sends:

```text
async_insert=1
wait_for_async_insert=1
```

Optional async settings supported by config:

- `async_insert_busy_timeout_ms`
- `async_insert_max_data_size`
- `async_insert_max_query_number`
- `async_insert_deduplicate`

## ClickHouse Topology

For local development and canary examples, plain `MergeTree` tables are enough.

For production:

- Prefer ClickHouse Cloud defaults when using ClickHouse Cloud.
- For self-managed deployments, start with one shard and add replicas for availability before adding shards.
- Use `ReplicatedMergeTree` or replicated databases with ClickHouse Keeper when the data cannot be cheaply recreated.
- Add a generated cluster-DDL mode later if self-managed sharded deployments need local replicated tables plus `Distributed` tables.

## Future Work

- Add renderer-controlled replicated/distributed DDL modes.
- Add explicit insert deduplication token support if retry idempotency needs to be stronger than deterministic landing IDs.
- Add serving/canonicalization tables on top of landing tables.
- Tune high-volume table partitioning and `ORDER BY` keys after measuring real workloads.
