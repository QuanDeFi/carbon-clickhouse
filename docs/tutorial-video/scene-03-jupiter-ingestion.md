# Scene 03: Jupiter Instruction And Event Ingestion

Now we run the Jupiter Swap ClickHouse example. The environment already provides
the provider RPC URL, ClickHouse endpoint, and bounded slot range, so the visible
command can stay focused on the example itself.

While the command runs, watch for the Carbon pipeline startup, the block crawler
processing the range, and the ClickHouse sink counters. The important signal is
that the process exits cleanly after inserting typed Jupiter instruction and
event rows.
