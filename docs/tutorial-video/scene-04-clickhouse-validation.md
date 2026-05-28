# Scene 04: Start The Token Program Live Run

Next we start the Token Program ClickHouse example in a second terminal window.
This keeps the two pipelines visually separate while both write into the same
ClickHouse instance and expose metrics to the same monitoring stack.

The Token Program example first writes fixed USDC account snapshots, then keeps
tailing finalized blocks for live Token Program instructions. With Jupiter and
Token Program now running together, the next scene can show both Carbon-side
metrics and ClickHouse-side query activity while ingestion is still happening.
