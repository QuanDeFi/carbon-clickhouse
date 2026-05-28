# Scene 05: Live Observability

With both examples running, we open Grafana first. This dashboard is the Carbon
view of the system: processed updates, queue depth, buffered rows, inserted
rows, retries, and backpressure. It answers whether the pipelines are keeping
up while live data is flowing.

Then we switch to ClickStack. ClickStack answers the database-side question:
what ClickHouse is doing while Carbon writes into it. The source is
system.query_log, so the viewer can compare query kinds, users, row counts, and
durations against the Carbon metrics from Grafana.
