# Scene 03: Start The Jupiter Live Run

Now we start the Jupiter Swap ClickHouse example. This is a live head-follow
run, so the process keeps reading finalized blocks instead of stopping after a
fixed slot range.

The command stays focused on the example package; the RPC URL and ClickHouse
endpoint are provided by the environment. As the logs move, the important thing
is that the Carbon pipeline starts, the block crawler follows head, and the
ClickHouse sink begins flushing decoded Jupiter rows. We leave this terminal
running so later dashboard scenes show live activity.
