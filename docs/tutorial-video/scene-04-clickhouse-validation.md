# Scene 04: Token Program Live Ingestion

Next we start the Token Program ClickHouse example in a second terminal window.
This keeps the Jupiter and Token Program examples visually separate while both
write to the same local ClickHouse and monitoring stack.

The Token Program example starts with fixed USDC account snapshots and then
keeps tailing finalized blocks for live Token Program instructions. With both
examples running, the next scenes can show Carbon-side metrics and ClickHouse-
side query activity while ingestion is still in progress.
