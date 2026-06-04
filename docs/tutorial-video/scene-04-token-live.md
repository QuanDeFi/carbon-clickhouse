We start the Token Program example in a second terminal.

It uses the same database with a separate metrics port. Startup prepares the
generated Token Program account and instruction tables.

The example first fetches four fixed USDC accounts with `getMultipleAccounts`
and writes those decoded account snapshots. Then it starts a live finalized
block crawler from the next slot, decoding Token Program instructions as they
appear.

Both example processes keep running in parallel.
