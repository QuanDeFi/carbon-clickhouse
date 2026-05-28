# Scene 06: Landing Table Inspection

Now that the live runs have produced rows, we inspect the generated landing
tables through ClickHouse Play. This is the built-in ClickHouse SQL UI, so it is
the simplest browser-based way to look directly at the data without adding
another database explorer.

The Jupiter queries show route and swap-event rows: slots, signatures, amounts,
slippage, route legs, AMMs, and execution amounts. The Token Program queries
show the account-family landing tables and USDC account snapshot fields such as
mint, token owner, amount, state, and mode. This ties the generated schema back
to real decoded Solana data.
