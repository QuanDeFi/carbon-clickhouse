# Scene 07: Async Insert Evidence

By default, the sink uses synchronous inserts. For live ingestion, callers can
opt into ClickHouse async inserts while still waiting for acknowledgement with
wait_for_async_insert set to 1.

Here we enable the async insert switch for a short example run. Then we inspect
the ClickHouse async insert log in Play and compare it with query-log activity
in ClickStack. The point is not to introduce a new pipeline mode; it is to show
where an operator can verify async insert behavior from ClickHouse itself.
