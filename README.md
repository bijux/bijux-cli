# bijux-core

<!-- bijux-core-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-core/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-core/workflows/release-crates/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml)
[![PyPI Publish](https://github.com/bijux/bijux-core/workflows/release-pypi/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-core?display_name=tag&label=release)](https://github.com/bijux/bijux-core/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-2%20packages-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-core)
[![Published packages](https://img.shields.io/badge/published%20packages-2-2563EB)](https://github.com/bijux/bijux-core/tree/main/crates)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-dag docs](https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-core` is the shared source tree for two public Bijux products:

- `bijux`, a command runtime for automation, interactive workflows, plugins,
  mounted apps, and structured diagnostics
- `bijux-dag`, a deterministic graph runner with validation, execution,
  replay, artifact inspection, and evidence-backed comparison

The repository also carries the internal support crates that keep those
products testable, documented, and releasable from one audited workspace.

## What Ships From This Repository

| Surface | Delivery form | What it is for |
| --- | --- | --- |
| `bijux` | Rust crate, Python distribution, release bundles | operator-facing command runtime with config, history, memory, plugins, REPL, and diagnostics |
| `bijux-dag` | Rust crates and release bundles | local DAG authoring, planning, execution, replay, diff, and artifact verification |
| `bijux-dev` | repository-internal crate and binaries | maintainer diagnostics, contracts, inventories, and release proof |

The current workspace release line is `0.4.0`.

## Package Families

<!-- bijux-core-package-map:generated:start -->
The public package families in this repository are:

| Package | Purpose | Links |
| --- | --- | --- |
| `bijux-cli` | Public release family for the `bijux` command runtime, spanning the Rust crate, Python distribution, and release bundle. | <a href="https://crates.io/crates/bijux-cli"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-cli?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-cli"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://pypi.org/project/bijux-cli/"><img alt="PyPI" src="https://img.shields.io/pypi/v/bijux-cli?label=PyPI&logo=pypi" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/"><img alt="Docs" src="https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli"><img alt="GHCR" src="https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
| `bijux-dag` | Public DAG release family for deterministic graph authoring, artifact identity, execution, and the stamped `bijux-dag` command bundle. | <a href="https://crates.io/crates/bijux-dag-cli"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-dag-cli?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-dag-cli"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-dag/"><img alt="Docs" src="https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag"><img alt="GHCR" src="https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-cli"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
<!-- bijux-core-package-map:generated:end -->

Within those families, the workspace currently contains:

- public `bijux` crates: `bijux-cli`
- public `bijux-dag` crates: `bijux-dag-core`, `bijux-dag-artifacts`,
  `bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-cli`
- repository-internal support crates: `bijux-cli-python`, `bijux-dag-testkit`,
  `bijux-dev`

The canonical package-boundary reference lives in
[`docs/bijux-core/foundation/package-boundary.md`](docs/bijux-core/foundation/package-boundary.md).

## Repository Layout

- [`crates/bijux-cli`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli): Rust runtime crate behind the `bijux` executable.
- [`crates/bijux-cli-python`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli-python): Python bridge package and native extension surface for CLI runtime distribution.
- [`crates/bijux-dag-core`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-core): DAG schema, invariants, canonicalization, hashing, and replay/diff primitives.
- [`crates/bijux-dag-runtime`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-runtime): DAG execution engine and run lifecycle behavior.
- [`crates/bijux-dag-app`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-app): DAG command orchestration, response modeling, and render flows.
- [`crates/bijux-dag-cli`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-cli): thin binary entrypoint for `bijux-dag`.
- [`crates/bijux-dag-artifacts`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-artifacts): artifact and persistence utilities for DAG evidence handling.
- [`crates/bijux-dag-testkit`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-testkit): repository-internal fixtures and helpers for DAG contract testing.
- [`crates/bijux-dev`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dev): maintainer control plane for governance, diagnostics, release contracts, and evidence tooling.
- [`docs/`](https://github.com/bijux/bijux-core/tree/main/docs): canonical handbook set for repository, CLI, DAG, and maintainer surfaces.
- [`makes/`](https://github.com/bijux/bijux-core/tree/main/makes): make modules for root workflows, Rust/Python validation, DAG commands, docs, and release automation.

## Command Surfaces

The operator-facing DAG contract is the visible `bijux-dag --help` surface.
That default surface covers validation, planning, execution, replay, run
inspection, artifact inspection, diff, verify, doctor, cache, and command
discovery. Repository-owned experimental routes remain callable by explicit
path, while modeled-platform and maintainer namespaces require
`BIJUX_DAG_ENABLE_SIMULATED=1` or `BIJUX_DAG_ENABLE_INTERNAL=1`.

Use the DAG handbook for the current route inventory and release boundary:

- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [`docs/bijux-dag/interfaces/cli-surface.md`](docs/bijux-dag/interfaces/cli-surface.md)
- [`docs/bijux-dag/foundation/release-boundary.md`](docs/bijux-dag/foundation/release-boundary.md)

### `bijux-dag` v0.4.0 Surface Truth Table

| Class | `v0.4.0` meaning | Representative surfaces |
| --- | --- | --- |
| stable | supported visible `bijux-dag --help` surface for local DAG authoring, execution, replay, and evidence inspection | `validate`, `plan`, `run`, `replay`, `runs ...`, `artifact`, `artifact-inspect`, `diff`, `explain`, `verify`, `doctor`, `cache`, `version`, `commands`, `completions` |
| experimental | callable by explicit path and repository-tested, but outside the stable operator compatibility lane | `init`, `canonicalize`, `graph`, `graph-lint`, `fingerprint`, `hash`, `status`, `node`, `trace-artifact`, `why-rerun`, `why-cache-missed`, `export`, `import`, `migrate`, `adapters`, `config`, `policy`, `fsck`, `prove`, `proof-summary` |
| simulated | modeled platform and control-plane namespaces that require `BIJUX_DAG_ENABLE_SIMULATED=1`, not production backends or services | `control-plane`, `state-store`, `dataset`, `enterprise`, `fleet`, `governance`, `federation`, `incident`, `lab` |
| internal | maintainer-only and contract-only routes that require `BIJUX_DAG_ENABLE_INTERNAL=1` and stay outside the public operator boundary | `security`, `durability`, `performance`, `release`, `runtime`, `schedule`, `version-inspect`, `capabilities`, `semantic-portability`, `equivalence-proof` |
| future | not a `v0.4.0` product promise | cluster-backed kubernetes execution, cluster-backed slurm or hpc execution, public remote workers, public enterprise or federation APIs, full scheduler service |

The canonical machine-readable release boundary is
`contracts/foundation/dag_release_truth_table.v1.json`.

## Quick Start

Local builds, CI, and release jobs all use the pinned Rust `1.86.0` toolchain
declared in `rust-toolchain.toml`.

Install the public command surfaces:

```bash
cargo install bijux-cli
cargo install bijux-dag-cli
python -m pip install bijux-cli
```

Build and test from repository root:

```bash
cargo check --workspace
cargo test --workspace
```

Inspect product command surfaces:

```bash
cargo run -p bijux-cli --bin bijux -- --help
cargo run -p bijux-dag-cli --bin bijux-dag -- --help
cargo run -p bijux-dag-cli --bin bijux-dag -- validate --help
cargo run -p bijux-dag-cli --bin bijux-dag -- commands --all
```

Try a local DAG flow against a repository fixture:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/authoring/examples/minimal_consumer.dag.json

cargo run -p bijux-dag-cli --bin bijux-dag -- run \
  evidence/authoring/examples/minimal_consumer.dag.json \
  --out artifacts/runs
```

## Documentation

- repository handbook:
  [bijux-core handbook](https://bijux.io/bijux-core/bijux-core/)
- CLI handbook:
  [bijux-cli handbook](https://bijux.io/bijux-core/bijux-cli/)
- DAG handbook:
  [bijux-dag handbook](https://bijux.io/bijux-core/bijux-dag/)
- maintainer handbook:
  [bijux-dev handbook](https://bijux.io/bijux-core/bijux-dev/)

If you are reading code and need the owning package before the owning command,
start with:

- [`docs/bijux-core/foundation/package-map.md`](docs/bijux-core/foundation/package-map.md)
- [`docs/bijux-dag/packages/index.md`](docs/bijux-dag/packages/index.md)
- [`docs/bijux-cli/packages/index.md`](docs/bijux-cli/packages/index.md)

## Maintainer Workflows

```bash
make help
make docs-check
make dag-test
make dag-contracts
```

This repository keeps public product code and release proof together so that
behavior, contracts, documentation, and evidence can drift only through
reviewed changes in the same tree.

## License

Apache-2.0 ([`LICENSE`](https://github.com/bijux/bijux-core/blob/main/LICENSE)).
