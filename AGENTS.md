# Agent Instructions for Carbon ClickHouse Demo Automation

## Scope

These instructions apply to the whole repository.

## Primary Objective

Maintain deterministic scripts for the ClickHouse sink tutorial recording. Do
not rely on autonomous UI or terminal behavior during final recording.

## Current Demo Source Of Truth

`scripts/demo/runbook.yaml`

## Primary Tutorial Source

`crates/core/src/clickhouse/docs/ClickHouse Sink Tutorial Curriculum.md`

## Examples

- `examples/jupiter-swap-clickhouse`
- `examples/token-program-clickhouse`

## Recording Architecture

- Headless GUI layer: Xvfb + x11vnc + noVNC.
- Browser UI: Playwright.
- Default recording backend: FFmpeg x11grab.
- Optional recording backend: OBS via obs-websocket.
- Voiceover: ElevenLabs `eleven_multilingual_v2`.
- Final assembly: FFmpeg.

## Safety

- Never print RPC URLs, API keys, OBS websocket passwords, or ElevenLabs secrets.
- Do not commit `.env.demo.local`.
- Do not touch `crates/core/src/clickhouse/docs/Branch Diff to v1.md` unless
  explicitly asked.
- Do not run destructive ClickHouse resets without
  `DEMO_ALLOW_CLICKHOUSE_RESET=true`.
- Do not stop existing demo/example processes without
  `DEMO_ALLOW_STOP_PROCESSES=true`.
- Do not use public Solana mainnet-beta RPC for tutorial smoke runs.

## Do Not

- Do not improvise commands during final recording.
- Do not rely on autonomous UI clicking.
- Do not alter root Node package setup unless absolutely necessary.
- Do not assume live-chain activity will produce tutorial rows. Prefer bounded
  slot ranges.
