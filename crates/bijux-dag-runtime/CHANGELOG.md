# Changelog

All notable changes to **bijux-dag-runtime** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## 0.4.0 – 2026-07-24

### Added
- First public crates.io release of `bijux-dag-runtime` as the execution engine
  behind `bijux-dag`.
- Stable and prelude import lanes for supported execution, cache, replay, and
  verification workflows.
- Deterministic plan execution with dependency scheduling, selector closure,
  retries, timeouts, and terminal node outcomes.
- Local process and container-backed execution boundaries with explicit
  capability and infrastructure failure reporting.
- Adapter contracts, registries, conformance checks, and external adapter
  integration without moving graph semantics into runtime code.
- Built-in constant, shell, file-transform, and container adapters for the
  supported local workflow surface.
- Policy evaluation and retained policy traces for declared effects and
  execution decisions.
- Content-addressed cache keys, cache proof verification, corruption refusal,
  and lineage-aware reuse records.
- Replay classification, retained-evidence verification, and difference
  reporting for focused reruns.
- Structured diagnostics, stable error classification, lifecycle events, and
  ordered execution timelines.
