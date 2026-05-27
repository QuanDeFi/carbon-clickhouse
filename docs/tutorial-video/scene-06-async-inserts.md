# Scene 06: ClickHouse Table Data Inspection

Now we inspect the generated landing tables through ClickHouse's built-in
`/play` UI. This is the ClickHouse-specific browser surface for running SQL and
looking directly at table contents.

The Jupiter queries show route rows and swap-event rows with slots, signature
prefixes, amounts, slippage, route-leg counts, AMMs, and execution amounts. The
Token Program queries show bootstrapped account-family tables and USDC account
snapshot data such as mint, token owner, amount, state, and mode.
