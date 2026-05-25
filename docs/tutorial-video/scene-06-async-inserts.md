# Scene 06: Async-Wait Inserts

The sink defaults to synchronous inserts. For live ingestion, callers can opt
into ClickHouse async inserts while still waiting for acknowledgement.

The source search shows where the async setting is wired. We then turn on the
environment switch, run a small token snapshot, and inspect ClickHouse's
asynchronous insert log. A successful row in that log proves that async insert
was used while the client still waited for ClickHouse to accept the batch.
