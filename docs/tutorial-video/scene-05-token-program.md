# Scene 05: Live Observability

With both examples running, we open Grafana first. Grafana shows Carbon-side
health: processed updates, queue depth, buffered rows, inserted rows, retries,
and backpressure. This answers whether the Carbon pipelines are keeping up.

Then we open ClickStack. ClickStack answers a different question: what
ClickHouse itself is doing while Carbon writes into it. It reads
`system.query_log`, so the viewer can see query kinds, users, row counts, and
durations from the database side.
