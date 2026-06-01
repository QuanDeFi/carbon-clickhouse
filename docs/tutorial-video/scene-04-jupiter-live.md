With setup and configuration identified, we start the Jupiter Swap example.

The command is intentionally plain because the env file already carries the
ClickHouse URL, RPC URL, slot mode, metrics port, and log level. Startup creates
or reconciles the generated Jupiter landing tables, connects to Solana RPC,
follows finalized blocks near head, decodes Jupiter instructions and events,
and writes generated rows into ClickHouse.

We leave it running so the later observability scene can show a live pipeline,
not a completed batch job.
