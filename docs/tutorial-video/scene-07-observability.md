# Scene 07: Async-Wait Inserts And Async Insert Log

The sink defaults to synchronous inserts. For live ingestion, callers can opt
into ClickHouse async inserts while still waiting for acknowledgement with
`wait_for_async_insert=1`.

We enable the async insert switch for a short example run, then inspect the
ClickHouse async insert log. In `/play`, the end user can query
`system.asynchronous_insert_log` directly. In ClickStack, the same database-side
activity is visible through `system.query_log`.
