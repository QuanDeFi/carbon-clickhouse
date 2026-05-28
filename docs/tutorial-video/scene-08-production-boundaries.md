# Scene 08: Production Boundaries

This final slide separates the responsibilities. The ClickHouse sink owns typed
landing ingestion, generated schema bootstrap, buffering, retries, async insert
settings, and sink metrics.

ClickHouse owns physical storage, sorting, merges, server-side async insert
batching, replicated or distributed topology, query logs, and downstream
serving tables.

Durable queues, replay coordination, checkpoints, dead-letter handling,
finality policy, and serving APIs belong outside the sink in an external
control plane. While this slide is visible, the live example terminals can stop
in the background.
