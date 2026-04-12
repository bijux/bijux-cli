# bijux-core

<!-- bijux-core-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml)
[![Docs](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml)
[![PyPI Publish](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-core?display_name=tag&label=release)](https://github.com/bijux/bijux-core/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-1%20package-181717?logo=github)](https://github.com/bijux?tab=packages)
[![Published packages](https://img.shields.io/badge/published%20packages-1-2563EB)](https://github.com/bijux/bijux-core/tree/main/crates)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli)

[![bijux-cli docs.rs](https://img.shields.io/docsrs/bijux-cli?label=bijux--cli%20docs.rs)](https://docs.rs/bijux-cli)

[![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/)

[![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli)

[![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/)
<!-- bijux-core-badges:generated:end -->

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

## Package Map

<!-- bijux-core-package-map:generated:start -->
The public package families in this repository are:

| Package | Purpose | Links |
| --- | --- | --- |
| `bijux-cli` | Public release family for the `bijux` command runtime, spanning the Rust crate, Python distribution, and release bundle. | <a href="https://crates.io/crates/bijux-cli"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-cli?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-cli"><img alt="Docs.rs" src="https://img.shields.io/docsrs/bijux-cli?label=docs.rs" height="18"></a> <a href="https://pypi.org/project/bijux-cli/"><img alt="PyPI" src="https://img.shields.io/pypi/v/bijux-cli?label=PyPI&logo=pypi" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/"><img alt="Docs" src="https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli"><img alt="GHCR" src="https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
<!-- bijux-core-package-map:generated:end -->

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
