We start the Token Program example in a second terminal.

Startup prepares the generated Token Program account and instruction tables.

The example first fetches four fixed USDC accounts with `getMultipleAccounts`
and writes those decoded account snapshots into ClickHouse.
Then it starts a live run with the
block crawler datasource from the next slot and decodes
Token Program instructions as they appear.
