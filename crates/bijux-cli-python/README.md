# bijux-core

`bijux-core` is the canonical repository for the Bijux platform.
It contains two real products built and governed together:

- `bijux-cli`: the operator-facing command runtime.
- `bijux-dag`: the deterministic graph execution and evidence system.

The goal of this repository is simple: ship both products from one audited source of truth, with clear boundaries, strong contracts, and release-grade traceability for both humans and machines.

## Products

| Product | What it does | Primary users | Runtime entrypoint |
|---|---|---|---|
| `bijux-cli` | Runs automation and interactive workflows with structured output, plugin routing, and stable command semantics. | Operators, developers, automation systems | `bijux` |
| `bijux-dag` | Defines, validates, executes, replays, and diffs computation graphs with deterministic artifact and evidence behavior. | DAG authors, platform teams, governance workflows | `bijux-dag dag ...` |

## Release State

Current release line: **`v0.3.4`**.

- `bijux-cli` is the active public release surface at `v0.3.4`.
- `bijux-cli-python` is the Python packaging bridge for the same CLI runtime.
- `bijux-dag` remains an internal workspace product until the coordinated public release target at `v0.4.0`.

## Repository Structure

- `crates/bijux-cli`: Rust runtime crate behind the `bijux` executable.
- `crates/bijux-cli-python`: Python bridge package and native extension surface for CLI runtime distribution.
- `crates/bijux-dag-core`: DAG schema, invariants, canonicalization, hashing, and replay/diff primitives.
- `crates/bijux-dag-runtime`: DAG execution engine and run lifecycle behavior.
- `crates/bijux-dag-app`: DAG command orchestration, response modeling, and render flows.
- `crates/bijux-dag-cli`: thin binary entrypoint for `bijux-dag`.
- `crates/bijux-dag-artifacts`: artifact and persistence utilities for DAG evidence handling.
- `crates/bijux-dag-testkit`: fixtures and helpers for DAG contract testing.
- `crates/bijux-dev`: maintainer control plane for governance, diagnostics, release contracts, and evidence tooling.
- `docs/`: canonical handbook set for repository, CLI, DAG, and maintainer surfaces.
- `makes/`: make modules for root workflows, Rust/Python validation, DAG commands, docs, and release automation.

## Quick Start

From repository root:

```bash
cargo check --workspace
cargo test --workspace
```

Inspect product command surfaces:

```bash
cargo run -p bijux-cli --bin bijux -- --help
cargo run -p bijux-dag-cli --bin bijux-dag -- --help
cargo run -p bijux-dag-cli --bin bijux-dag -- dag --help
```

## Maintainer Workflows

```bash
make help
make dag-help
make dag-test
make dag-contracts
```

## Documentation Map

- Repository handbook: `docs/bijux-core/`
- CLI handbook: `docs/bijux-cli/`
- DAG handbook: `docs/bijux-dag/`
- Maintainer handbook: `docs/bijux-dev/`
- Release history: `CHANGELOG.md`

## Why Unified Ownership

`bijux-cli` and `bijux-dag` are developed as separate products with explicit ownership boundaries, but they share one governance and release backbone.
This repository model keeps:

- product behavior reviewable at crate boundaries,
- compatibility decisions tied to code and tests,
- release evidence and documentation aligned with tagged source.

## License

Apache-2.0 (`LICENSE`).
