Finally, ClickHouse Play validates the data directly.

The database menu shows generated landing tables with row and byte metadata, so
we can see which decoder-owned tables were created and which ones are
populated.

We inspect Jupiter event rows and route instructions, then Token Program checked transfers and a multisig account snapshot.

The Jupiter event rows show swap legs: AMMs, input and output mints, and
amounts. The route instruction rows show the route request, quoted output,
slippage guard, and source mode.

The Token Program rows show checked transfer amounts and decimals, plus account
authority state from the multisig snapshot.

That gives us the three generated table families in one place: event,
instruction, and account rows. Table metadata plus decoded rows validates the
pipeline end to end.
