With setup and configuration identified, we start the Jupiter Swap example.

This command reads the env, creates or reconciles the generated Jupiter landing
tables, connects to Solana RPC, follows finalized blocks near head, decodes
Jupiter instructions and events, and writes generated rows into ClickHouse.

We leave it running so the later observability scene can show a live pipeline,
not a completed batch job.
