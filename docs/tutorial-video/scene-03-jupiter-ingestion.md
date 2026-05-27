# Scene 03: Jupiter Instruction And Event Ingestion

Now we start the Jupiter Swap ClickHouse example as a live head-follow run. The
environment already provides the provider RPC URL and ClickHouse endpoint, so
the visible command can stay focused on the example itself.

While the command runs, watch for the Carbon pipeline startup, the block crawler
following finalized blocks, and the ClickHouse sink counters. We leave this
process running so the dashboard scenes can show live Carbon and ClickHouse
activity instead of only post-run logs.
