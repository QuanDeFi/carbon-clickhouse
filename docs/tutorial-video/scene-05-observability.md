With both examples active, Grafana shows the Carbon-side pipeline view.

Processed updates show throughput. Failed updates should stay quiet. Queue
depth and process latency show whether the pipeline is keeping up with live
blocks.

The ClickHouse panels show buffered rows, buffered bytes, active buffers,
inserted rows, and flush failures. Together they tell us whether Carbon is
decoding, buffering, and flushing successfully.

The Token Program dashboard follows the same layout for the second example.

ClickStack then gives the database-side view. The Inserts dashboard shows rows
and bytes arriving per generated landing table, so it complements Grafana by
showing what ClickHouse itself is receiving.
