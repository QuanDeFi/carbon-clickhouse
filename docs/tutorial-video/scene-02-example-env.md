Next we take a look at where the example configuration lives.

The env files set the database URL, RPC URL, metrics ports, log level, slot range, and the optional async-wait insert switch.

Leaving the start and end slots empty will put the example into live mode.

For this tutorial we will run two pipelines in parallel and enable async-wait inserts,

which allow ClickHouse to batch inserts from multiple concurrent writers server-side.

If you only run one pipeline, you can leave it in default sync mode.
