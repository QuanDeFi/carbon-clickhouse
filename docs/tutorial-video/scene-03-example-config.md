Now we connect the tutorial env values to the core ClickHouse sink configuration.

The example `.env.example` files show the runtime inputs the viewer cares
about before any command runs: `DATABASE_URL` for ClickHouse, `RPC_URL` for
Solana, the Jupiter slot mode, each Prometheus metrics port, and `LOG_LEVEL`.
That lets the terminal commands stay clean later.

Then we move into the core sink code instead of the example wiring. The config
module is the reference surface: insert mode, batch sizing, transport behavior,
retry policy, and deduplication all live here. `ClickHouseConfig` holds the
endpoint, database, credentials, table, source name, mode, decoder version, and
the nested settings.

The URL helper turns `DATABASE_URL` into endpoint and auth fields. The builder
methods are where a caller overrides defaults. The writer consumes the same
config for retries, async insert query settings, and dedup tokens, while the
admin path uses it for generated schema setup before rows land in ClickHouse.
