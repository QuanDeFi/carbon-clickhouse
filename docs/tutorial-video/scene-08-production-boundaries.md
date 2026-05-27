# Scene 08: Production Boundaries

The ClickHouse sink handles typed landing ingestion, generated schema bootstrap,
buffering, retries, async-wait insert settings, and sink metrics.

ClickHouse handles physical storage, sorting, merges, server-side async insert
batching, replicated or distributed topology, query logs, and downstream
serving tables.

Durable queues, replay coordination, checkpoints, DLQs, finality policy, and
serving APIs belong outside the sink in an external control plane. While this
slide is visible, the live example terminals can be stopped in the background.
