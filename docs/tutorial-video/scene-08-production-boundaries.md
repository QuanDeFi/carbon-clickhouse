# Scene 08: Production Boundaries

The ClickHouse sink handles typed landing ingestion, buffering, retries, schema
bootstrap, and metrics. Durable queues, replay coordination, canonical serving
tables, and finality policy belong outside the sink.
