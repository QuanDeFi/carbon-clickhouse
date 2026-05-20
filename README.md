<div align="center">
  <h1>Carbon</h1>
  <p><strong>Rust framework for building Solana indexers and data pipelines.</strong></p>

  <p>
    <a href="https://crates.io/crates/carbon-core"><img alt="Crates.io" src="https://img.shields.io/crates/v/carbon-core"></a>
    <a href="https://www.npmjs.com/package/@sevenlabs-hq/carbon-cli"><img alt="npm" src="https://img.shields.io/npm/v/@sevenlabs-hq/carbon-cli"></a>
    <a href="LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-blue"></a>
  </p>

  <p>
    <a href="#-overview">Overview</a> ·
    <a href="#-quick-start">Quick Start</a> ·
    <a href="#-capabilities">Capabilities</a> ·
    <a href="#-maintained-packages">Packages</a> ·
    <a href="#-examples">Examples</a>
  </p>
</div>

---

## ✨ Overview

Carbon is built around a pipeline: datasources stream Solana updates into the
runtime, decoders turn raw account and transaction data into typed Rust
structures, and processors handle the decoded output in order to build data
pipelines and indexers for your applications. Each part is modular, so you can
swap RPC streams for Geyser streams, generated decoders for custom ones, or
logging processors for Postgres, GraphQL, analytics, bots, and other
application-specific sinks.

---

## 🚀 Quick Start

Start with a decoder. Use one of Carbon's existing program decoders, or generate
one from your program IDL:

```sh
npx @sevenlabs-hq/carbon-cli parse \
  --idl ./idl.json \
  --out-dir ./my-program-decoder \
  --name my-program
```

Then add a datasource and wire them together in a pipeline:

```rust
use carbon_core::{error::CarbonResult, pipeline::Pipeline};
use carbon_log_metrics::LogMetrics;
use carbon_my_program_decoder::{MyProgramDecoder, PROGRAM_ID};
use carbon_rpc_block_subscribe_datasource::{Filters, RpcBlockSubscribe};
use solana_client::rpc_config::RpcBlockSubscribeFilter;

#[tokio::main]
async fn main() -> CarbonResult<()> {
    let datasource = RpcBlockSubscribe::new(
        "wss://api.mainnet-beta.solana.com".to_string(),
        Filters::new(
            RpcBlockSubscribeFilter::MentionsAccountOrProgram(PROGRAM_ID.to_string()),
            None,
        ),
    );

    Pipeline::builder()
        .datasource(datasource)
        .metrics(std::sync::Arc::new(LogMetrics::new()))
        .instruction(MyProgramDecoder, MyProcessor)
        .build()?
        .run()
        .await
}
```

For real-world usage patterns, see [`examples/`](examples/) — each example is a complete working pipeline.

---

## Generating Decoders from IDL

Decoders convert raw Solana account or instruction data into strongly typed Rust structures.

Carbon includes a CLI that generates decoders from:

- Anchor IDLs
- Codama IDLs
- On-chain program addresses

### CLI Installation

```sh
npm install -g @sevenlabs-hq/carbon-cli
```

Or run directly:

```sh
npx @sevenlabs-hq/carbon-cli
```

### CLI Usage

```sh
carbon-cli parse [OPTIONS]
carbon-cli scaffold [OPTIONS]
```

#### Parse Options

- `-i, --idl <fileOrAddress>`: Path to an IDL json file or a Solana program address
- `-o, --out-dir <dir>`: Output directory for generated code
- `-c, --as-crate`: Generate as a Cargo crate layout
- `-s, --standard <anchor|codama>`: Specify the IDL standard to parse (default: anchor)
- `--event-hints <csv>`: Comma-separated names of defined types to parse as CPI Events (Codama only)
- `-u, --url <rpcUrl>`: RPC URL for fetching IDL when using a program address
- `--with-clickhouse <boolean>`: Include ClickHouse wiring and deps (default: false)
- `--clickhouse-options <jsonOrFile>`: Renderer ClickHouse options as a JSON object or path to a JSON file
- `--no-clean`: Do not delete output directory before rendering

#### Scaffold Options

- `-n, --name <string>`: Name of your project
- `-o, --out-dir <dir>`: Output directory
- `-d, --decoder <name>`: Decoder name (auto-detected from IDL)
- `--idl <fileOrAddress>`: IDL file or program address
- `--idl-standard <anchor|codama>`: IDL standard
- `--idl-url <rpcUrl>`: RPC URL for fetching IDL (when using program address)
- `--event-hints <csv>`: Event hints for Codama IDL
- `-s, --data-source <name>`: Name of data source
- `-m, --metrics <log|prometheus>`: Metrics to use (default: log)
- `--with-postgres <boolean>`: Include Postgres wiring and deps (default: true)
- `--with-graphql <boolean>`: Include GraphQL wiring and deps (default: true)
- `--with-clickhouse <boolean>`: Include ClickHouse wiring and deps (default: false)
- `--clickhouse-options <jsonOrFile>`: Renderer ClickHouse options as a JSON object or path to a JSON file
- `--with-serde <boolean>`: Include serde feature for decoder (default: false)
- `--force`: Overwrite output directory if it exists

### Example: Generate Decoder from Program

```sh
carbon-cli parse \
  --idl LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo \
  --url mainnet-beta \
  --out-dir ./decoders
```

### Example: Scaffold Project

```sh
carbon-cli scaffold \
  --name my-indexer \
  --idl ./idl.json \
  --data-source yellowstone-grpc
```

### Example: Generate Decoder With ClickHouse Production DDL Options

```sh
carbon-cli parse \
  --idl my_program.json \
  --out-dir ./src/decoders \
  --with-clickhouse true \
  --clickhouse-options '{"ddlMode":"replicated-merge-tree","onCluster":"main","ttl":{"instruction":"partition_time + INTERVAL 30 DAY"}}'
```

---

## Implementing Processors

Processors are async handlers that receive typed data from the pipeline.

```rust
use carbon_core::{
    error::CarbonResult,
    instruction::InstructionProcessorInputType,
    processor::Processor,
};
use carbon_my_program_decoder::instructions::MyProgramInstruction;

struct MyProcessor;

impl Processor<InstructionProcessorInputType<'_, MyProgramInstruction>>
    for MyProcessor
{
    async fn process(
        &mut self,
        input: &InstructionProcessorInputType<'_, MyProgramInstruction>,
    ) -> CarbonResult<()> {
        log::info!("instruction: {:?}", input.decoded_instruction);
        Ok(())
    }
}
```

The same pattern works with any decoder and any datasource: import the decoder
for the Solana program you care about, import the datasource that matches your
latency or backfill requirements, then implement a processor for the typed data
you want to handle.

---

## 🧰 Capabilities

- Stream transactions, account updates, account deletions, and block metadata.
- Decode instructions, nested CPIs, accounts, and emitted events into typed Rust
  structures.
- Build real-time indexers, historical backfills, snapshot loaders, and hybrid
  data pipelines with the same processor model.
- Combine multiple datasources and route updates with filters.
- Generate decoder crates from Anchor or Codama IDLs.
- Persist decoded data with typed Postgres rows or generic JSONB processors.
- Expose indexed data through GraphQL using the built-in Juniper and Axum
  helpers.
- Export pipeline metrics through logs or Prometheus.

---

## 📦 Maintained Packages

Carbon includes maintained datasources and decoders so most indexers can start
from existing building blocks instead of custom ingestion or decoding code.

### 🔌 Datasources

Datasource crates cover the common ways Solana data is consumed:

- **Geyser streams** for low-latency production indexers.
- **Solana RPC** for simple setups and public RPC compatibility.
- **Historical and snapshot sources** for backfills, range replay, and loading
  current account state.
- **Hosted provider APIs** (e.g. Helius) for provider-specific streaming and
  historical access.
- **Adapter datasources** for bringing your own message stream.

See [`datasources/`](datasources) for the full list.

### 🧬 Decoders

Decoder crates cover widely used Solana programs across core SPL programs,
DeFi, NFTs, infrastructure, and more. They expose typed accounts, instructions,
events, and optional serde, Postgres, and GraphQL integrations.

Use an existing decoder when one is available, or generate one from an Anchor or
Codama IDL with `carbon-cli parse`.

See [`decoders/`](decoders) for the full list.

---

## 🧪 Examples

The [`examples/`](examples) directory contains runnable indexers for the common
ways Carbon is used: Geyser streaming, public RPC streaming, transaction
backfills, account-state loading, validator snapshots, custom datasources, and
Postgres-backed GraphQL APIs.

Each example is a workspace crate and can be run from the repository root:

```sh
cargo run --release -p block-subscribe-rpc-carbon-example
```

---

## 🤝 Contributing

Contributions are welcome. See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the
development setup, local checks, and contribution guidelines.

## 📄 License

Carbon is licensed under the [MIT license](LICENSE).
