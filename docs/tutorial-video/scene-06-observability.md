With both examples active, we validate the running pipelines through the
observability stack.

Grafana is the Carbon-side view. We show the Jupiter dashboard first, then the
Token Program dashboard, so each running pipeline has its own metric surface:
processed updates, buffered rows, flushes, failures, retries, and other sink
metrics. ClickStack is the ClickHouse-side view: insert/query activity, rows
written, timing, and query-log evidence.

This scene proves more than “the terminals are printing logs.” It shows the two
live pipelines through structured runtime telemetry.
