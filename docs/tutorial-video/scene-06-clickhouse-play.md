Finally, ClickHouse Play validates the data directly.

The database menu shows landing tables with row and byte metadata. This shows which tables were created and populated.

Before looking at the table data, we group the tables into three families. Those families are accounts, instructions, and CPI events.

Account tables store account state. Instruction tables store decoded program calls. CPI event tables store event activity emitted through inner instructions.

Then we open the largest populated landing table: Token Program checked transfers.

Those rows show amount, decimals, slot, and signature. They also show instruction path and ingest context.

That gives us the final proof: ClickHouse reports rows and bytes. Decoded data arrives in the landing tables. The pipeline works end to end.
