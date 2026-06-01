Now we connect the tutorial env values to the core ClickHouse sink configuration.

The example READMEs show the two runtime inputs the viewer cares about:
`DATABASE_URL` for ClickHouse and `RPC_URL` for Solana. The RPC value proves
where live chain data comes from; the database value tells the sink where to
write.

Then we move into the core sink code instead of the example wiring. The config
module is the reference surface: insert mode, batch sizing, transport behavior,
retry policy, and deduplication all live here. `ClickHouseConfig` holds the
endpoint, database, credentials, table, source name, mode, decoder version, and
the nested settings.

The URL helper turns `DATABASE_URL` into endpoint and auth fields. The builder
methods are where a caller overrides defaults. The writer consumes the same
config for retries, async insert query settings, and dedup tokens, while the
admin path uses it for generated schema setup before rows land in ClickHouse.
