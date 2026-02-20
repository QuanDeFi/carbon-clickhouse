## Jupiter Swap → Postgres Pipeline Example

This example wires a Carbon pipeline that:

1. Listens to Jupiter swap transactions on Solana mainnet using either the `RpcTransactionCrawler` (default) or `RpcBlockCrawler` datasource.
2. Decodes swap instructions with `carbon-jupiter-swap-decoder`.
3. Persists decoded instructions into decoder-generated Postgres tables using Carbon's built-in Postgres processor and migrations.

The goal of this document is to walk through the full setup (from Docker + Postgres to `.env` wiring) and describe exactly which tables and columns are populated.

#### Pipeline Overview

```bash
+---------------------------+
|  Jupiter Program (JUP6L…) |
|  Swap instructions on     |
|  Solana mainnet           |
+-------------+-------------+
              |
              |  getSignaturesForAddress /
              |  getTransaction / getSlot / getblock
              v
+-------------+-------------+
|        Solana RPC         |
|         endpoint          |
+------+------+-------------+
       |      |
       |      |
       |      +---------------------------+
       |                                  |
       v                                  v
+------+---------------+        +---------+--------------+
| RpcTransactionCrawler|        | RpcBlockCrawler        |
| (historical polling) |        | (live block streaming) |
+----------+-----------+        +-----------+------------+
           |                                |
           +--------------+-----------------+
                          |
                          v
            +-------------+--------------+
            | carbon-jupiter-swap-decoder|
            |  + PostgresInstruction...  |
            +-------------+--------------+
                          |
                          v
            +-------------+--------------+
            | Decoder postgres wrappers  |
            | + built-in migrations      |
            +-------------+--------------+
                          |
                          v
   +----------------------+------+------------------------------+
   | Postgres (Docker)                                        |
   | Tables populated:                                        |
   |  • route_instruction / route_v2_instruction              |
   |  • exact_out_route_instruction (+ v2)                    |
   |  • shared_accounts_* route tables                        |
   |  • cpi_events                                             |
   +----------------------------------------------------------+
```

---

### 1. Prerequisites

| Requirement | Why it is needed |
|-------------|------------------|
| **Ubuntu** | Same OS used for testing |
| **Docker** | To run Postgres locally without polluting your host system. |
| **Postgres** | (Via Docker) used to store the decoded data. |
| **Rust toolchain (1.82+)** | Installs `rustc` and `cargo`, which build and run the example binary. |
| **System libs: `build-essential`, `pkg-config`, `libssl-dev`** | Needed by sqlx and other crates that compile native dependencies. |
| **Access to a Solana RPC endpoint** | The crawler relies on HTTP `getSignaturesForAddress`, `getTransaction`, `getSlot` and `getBlock` . |

Install Docker first (https://docs.docker.com/get-docker/). Once Docker is running you can provision a Postgres container.

---

### 2. Provision Postgres with Docker

```bash
# Creates persistent storage for Postgres
docker volume create pgdata

# Pulls the Postgres container and runs it locally, exposing port 5432
docker run -d --name pg \
  -e POSTGRES_USER=USERNAME \
  -e POSTGRES_PASSWORD=PASSWORD \
  -e POSTGRES_DB=DATABASE_NAME \
  -p 127.0.0.1:5432:5432 \
  -v pgdata:/var/lib/postgresql/data \
  --health-cmd='pg_isready -U USERNAME || exit 1' \
  postgres:17

# (Optional) to connect with psql via docker exec for admin & query tasks
docker exec -it pg psql -U USERNAME -d DATABASE_NAME
```

> **Notes**
> - The environment values (`USERNAME` / `PASSWORD` / `DATABSE_NAME`) must match what you put into `.env`.
> - The `pg-data` volume keeps data between container restarts. Remove the volume to wipe the DB.

---

### 3. Wire Environment Variables

Inside `examples/jupiter-swap-postgres/.env` you need:

```env
DATABASE_URL=postgres://USERNAME:PASSWORD@127.0.0.1:5432/DATABASE_NAME
RPC_URL=https://<your-rpc-provider-url>
DATASOURCE=rpc_transaction_crawler # or rpc_block_crawler
# RATE_LIMIT is optional, defaults to 10 requests/sec to work with free tier RPC nodes
RATE_LIMIT=10
```

* `DATABASE_URL` must line up with the container credentials `postgresql://[user[:password]@][host][:port][/dbname]`.
* `RPC_URL` should be an HTTPS endpoint that allows `getSignaturesForAddress/getTransaction`. (Triton, Alchemy, Helius etc.)
* `DATASOURCE` toggles which Carbon datasource powers the pipeline. Leave it at `rpc_transaction_crawler` for the historical/default behavior (signature polling), or switch to `rpc_block_crawler` to stream blocks as they are produced. The block crawler automatically starts from the most recent confirmed slot.
* `RATE_LIMIT` throttles both data sources to 10 requests/s by default to work with free tier RPC nodes.

The binary loads this `.env` file via `dotenv::dotenv().ok();` so you do **not** need to export anything globally.

---

### 4. Run the Example

```bash
cd examples/jupiter-swap-postgres
cargo run
```

What happens during startup:

1. Logging is set up (stdout + rotating `/logs/run-*.log` files).
2. `JupiterSwapInstructionsMigration` creates decoder-provided instruction tables if they do not already exist.
3. The datasource selected via `DATASOURCE` is initialized:
   - `RpcTransactionCrawler` polls for signatures mentioning the Jupiter program (`JUP6L...`) when backfilling via HTTP.
   - `RpcBlockCrawler` fetches the most recent slot via `getSlot` and immediately begins streaming new blocks forward using the same ~10 req/s pacing.
4. For each decoded instruction, `PostgresInstructionProcessor<JupiterSwapInstruction, JupiterSwapInstructionWithMetadata>` upserts into the appropriate decoder table.

You can inspect progress / warnings via the generated log file or by watching stdout. To confirm data is flowing, connect with `psql` and query the tables listed below.

---

### 5. Data Model

Once the pipeline runs you should see decoder-generated instruction tables, including:

- `route_instruction`
- `route_v2_instruction`
- `route_with_token_ledger_instruction`
- `exact_out_route_instruction`
- `exact_out_route_v2_instruction`
- `shared_accounts_route_instruction`
- `shared_accounts_route_v2_instruction`
- `shared_accounts_route_with_token_ledger_instruction`
- `shared_accounts_exact_out_route_instruction`
- `shared_accounts_exact_out_route_v2_instruction`
- `claim_instruction`
- `claim_token_instruction`
- `close_token_instruction`
- `create_token_account_instruction`
- `create_token_ledger_instruction`
- `set_token_ledger_instruction`
- `cpi_events`

Each table includes instruction metadata columns (`__signature`, `__instruction_index`, `__stack_height`, `__slot`) and decoded instruction payload fields for that variant.

---

### 6. Typical Workflow

1. **Start Postgres** (`docker start pg`).
2. **Confirm RPC credentials** are valid (e.g., curl `{"jsonrpc":"2.0","id":1,"method":"getHealth"}`).
3. **Adjust `.env`** if you want a different `DATASOURCE`, `RATE_LIMIT`, database host, or RPC endpoint.
4. **Run the pipeline** (`cargo run`). Leave it running to keep ingesting swaps; restart as needed.
5. **Inspect data** with `psql` queries such as:
   ```sql
   SELECT COUNT(*) FROM route_instruction;
   SELECT name, COUNT(*) FROM cpi_events GROUP BY 1 ORDER BY 2 DESC;
   ```

---

### 7. Troubleshooting

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| `Failed to run migrations: password authentication failed` | `.env` creds do not match container env | Update `DATABASE_URL` or recreate the container with matching `POSTGRES_*` values. |
| RPC related warnings or errors in logs | RPC rate-limiting or methods not supported | Lower `RATE_LIMIT`, upgrade RPC plan or change RPC provider. |
| Pipeline starts but stays idle with little/no output (`0 processed`) | Free-tier RPC plan may restrict methods required by `rpc_transaction_crawler` and/or `rpc_block_crawler` | Upgrade the RPC plan or switch to another RPC provider that allows required methods. |
| Block crawler logs `Failed to fetch the most recent slot...` | Temporary RPC hiccup retrieving `getSlot` | Re-run after the RPC endpoint recovers or switch back to `rpc_transaction_crawler`. |
| `0 processed` forever | RPC endpoint not returning Jupiter transactions | Verify RPC connection, run `curl getSignaturesForAddress JUP6...`, upgrade RPC plan or switch providers. |

This README should give you everything needed to run, debug, and understand the data emitted by `examples/jupiter-swap-postgres`.

#### Contact
Email: q@quandefi.co \
Telegram: [@QuanDeFi](https://t.me/QuanDeFi)
