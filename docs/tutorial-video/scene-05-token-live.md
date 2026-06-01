Now we start the Token Program example in a second terminal.

It first fetches the hardwired USDC account canary set through RPC, decodes those
Token Program accounts, and writes generated account landing rows. Then it
starts the live finalized block crawler for Token Program instructions and
writes generated instruction rows.

At this point both examples are running against the same ClickHouse instance and
both are exposing metrics for Prometheus.
