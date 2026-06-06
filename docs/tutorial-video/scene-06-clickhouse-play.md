Finally, ClickHouse Play validates the data directly.

The database menu shows landing tables with row and byte metadata. This shows which tables were created and populated.

Before looking at the table data, we group each example's tables into three families: accounts, instructions, and CPI events.

Account tables store account state. Instruction tables store decoded program calls. CPI event tables store event activity emitted through inner instructions.

Then we show the largest populated landing tables by rows and bytes.

Then we open the largest one: Token Program checked transfers.

For a stablecoin such as USDC, checked transfers are wallet-to-wallet sends, user deposits into protocols, or swap settlements.

As we scroll across the columns, ClickHouse is showing decoded token movement activity from the live pipeline.

That gives us the final proof: ClickHouse reports rows and bytes. Decoded data arrives in the landing tables. The pipeline works end to end.
