# Scene 07: Prometheus Observability

Carbon exposes process and ClickHouse sink metrics. In this rehearsal we use
Prometheus directly so the scene always shows real data instead of an empty
dashboard or a failed login screen.

The query is selected only after a pre-check confirms it has data. The value on
screen summarizes recent processed updates, while related metrics can show queue
depth, buffered rows, and ClickHouse flush failures.
