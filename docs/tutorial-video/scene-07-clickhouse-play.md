Finally we verify the landing rows directly in ClickHouse Play.

The Jupiter queries show generated Jupiter landing tables and sample decoded
rows. The Token Program queries show generated account and instruction landing
tables, including account fields from the USDC canary snapshot.

This is the end-state proof: the ClickHouse config was accepted, the RPC input
was accepted, decoding happened, generated rows were written, and the rows are
queryable in ClickHouse.
