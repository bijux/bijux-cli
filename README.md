# bijux-core

<!-- bijux-core-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-core/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-core/workflows/release-crates/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml)
[![PyPI Publish](https://github.com/bijux/bijux-core/workflows/release-pypi/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-core?display_name=tag&label=release)](https://github.com/bijux/bijux-core/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-2%20packages-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-core)
[![Published packages](https://img.shields.io/badge/published%20packages-6-2563EB)](https://github.com/bijux/bijux-core/tree/main/crates)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag-artifacts](https://img.shields.io/crates/v/bijux-dag-artifacts?label=artifacts&logo=rust)](https://crates.io/crates/bijux-dag-artifacts) [![bijux-dag-core](https://img.shields.io/crates/v/bijux-dag-core?label=core&logo=rust)](https://crates.io/crates/bijux-dag-core) [![bijux-dag-runtime](https://img.shields.io/crates/v/bijux-dag-runtime?label=runtime&logo=rust)](https://crates.io/crates/bijux-dag-runtime) [![bijux-dag-app](https://img.shields.io/crates/v/bijux-dag-app?label=app&logo=rust)](https://crates.io/crates/bijux-dag-app) [![bijux-dag-cli](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag-cli](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-dag-artifacts docs](https://img.shields.io/badge/docs-artifacts-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/) [![bijux-dag-core docs](https://img.shields.io/badge/docs-core-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-core/) [![bijux-dag-runtime docs](https://img.shields.io/badge/docs-runtime-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-runtime/) [![bijux-dag-app docs](https://img.shields.io/badge/docs-app-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-app/) [![bijux-dag-cli docs](https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag-artifacts docs.rs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts) [![bijux-dag-core docs.rs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-core) [![bijux-dag-runtime docs.rs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime) [![bijux-dag-app docs.rs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app) [![bijux-dag-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-core` is the repository behind two public Bijux products:

- `bijux`, the root command runtime for apps, plugins, configuration,
  diagnostics, and interactive workflows
- `bijux-dag`, the local-first DAG toolchain for graph validation, planning,
  execution, replay, artifact inspection, and verification

The same tree also carries the internal crates, contracts, docs, and release
automation that keep those products coherent. The public story is simple:

- use `bijux` when you want a general command runtime with mounted apps,
  plugins, config, history, memory, diagnostics, and REPL support
- use `bijux-dag` when you want reproducible local workflow execution with
  explicit graph contracts, retained run evidence, cache explanation, replay,
  and comparison

The current workspace release line is `0.4.0`.

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles.

## What Ships Today

| Surface | Delivery form | What it is for |
| --- | --- | --- |
| `bijux` | Rust crate, Python distribution, release bundles | operator-facing command runtime with apps, plugins, config, history, memory, REPL, and diagnostics |
| `bijux-dag` | Rust crates and release bundles | local-first DAG runtime for reproducible workflows with explicit graph contracts, deterministic execution records, verified artifacts, cache explanation, and replayable run bundles |
| `bijux-dev` | repository-internal crate and binaries | maintainer diagnostics, contracts, inventories, and release proof |

## Stable Product Boundary

`v0.4.0` is a real local product line today, but not every repository-owned
route is a public compatibility promise.

- `bijux` supports the visible root command surface shown by `bijux --help`,
  including runtime health, app and plugin routing, layered config, history,
  memory, and REPL workflows.
- `bijux-dag` supports the visible `bijux-dag --help` surface for local DAG
  work: `validate`, `plan`, `run`, `replay`, `runs`, `artifact`,
  `artifact-inspect`, `diff`, `explain`, `verify`, `doctor`, `cache`,
  `version`, `commands`, and `completions`.
- Branch-backed DAG workflows are part of the stable local operator surface.
  Retained runs record the selected branch decision, skipped lane, and join
  trigger outcome.
- Container-backed DAG nodes are part of the stable local operator surface
  when a supported engine such as Docker is available on `PATH`. Retained runs
  record engine and image identity and fail clearly when the engine is
  unavailable.
- `bijux-dag run --backend slurm` and `bijux-dag run --backend kubernetes`
  are part of the current release boundary with the documented shared-storage
  requirements for retained run evidence.
- Experimental DAG routes remain callable by explicit path, but they are not
  part of the stable compatibility lane. Use
  `bijux-dag commands --lane experimental` when you intentionally need that
  inventory.
- Simulated and maintainer-only DAG namespaces require deliberate opt-in
  through `BIJUX_DAG_ENABLE_SIMULATED=1` or `BIJUX_DAG_ENABLE_INTERNAL=1`
  together with lane inventory through `bijux-dag commands --lane simulated`
  or `bijux-dag commands --lane internal`.
- The repository-owned schedule and backfill lanes are tested and documented,
  but they remain internal surfaces rather than part of the default `v0.4.0`
  operator contract.
- Generic HPC execution beyond the shared-filesystem SLURM lane, public remote
  workers, and public enterprise or federation APIs are not part of the
  `v0.4.0` public product boundary.

### `bijux-dag` v0.4.0 Surface Truth Table

The canonical release-boundary contract for `bijux-dag` lives in
[`contracts/foundation/dag_release_truth_table.v1.json`](contracts/foundation/dag_release_truth_table.v1.json).
Use that file and the DAG handbook
[`docs/bijux-dag/foundation/release-boundary.md`](docs/bijux-dag/foundation/release-boundary.md)
when the release question is whether a route is stable, experimental,
simulated, internal, or still future work.
For what comes after that boundary, use
[`docs/tracking/bijux-dag-roadmap.md`](docs/tracking/bijux-dag-roadmap.md).

| Class | `v0.4.0` meaning | Representative surfaces |
| --- | --- | --- |
| stable | supported visible `bijux-dag --help` surface for local DAG authoring, execution, replay, and evidence inspection | `validate`, `plan`, `run`, `replay`, `runs ...`, `artifact`, `artifact-inspect`, `diff`, `explain`, `verify`, `doctor`, `cache`, `version`, `commands`, `completions` |
| experimental | callable by explicit path and repository-tested, but outside the stable operator compatibility lane | explicit-path operator helpers such as `init`, `status`, `export`, `migrate`, `prove`, and `trace-artifact`; use `bijux-dag commands --lane experimental` for the current inventory |
| simulated | modeled platform namespaces that require `BIJUX_DAG_ENABLE_SIMULATED=1`, not production backends or services | modeled control-plane and organizational route families; use `bijux-dag commands --lane simulated` only when you intentionally need repository-owned modeling surfaces |
| internal | maintainer-only and contract-only routes that require `BIJUX_DAG_ENABLE_INTERNAL=1` and stay outside the public operator boundary | maintainer verification, schedule, runtime, release, and capability lanes; use `bijux-dag commands --lane internal` only for deliberate repository maintenance work |
| future | not a `v0.4.0` product promise | generic hpc execution beyond the shared-filesystem slurm lane, public remote workers, public enterprise or federation APIs, full scheduler service |

Build operator procedures on the stable row. Treat the other rows as deliberate
opt-in surfaces, not as default product guarantees. For execution evidence,
backend requirements, and replay details, use the DAG handbook instead of
treating the root README as the full operating manual.

## Start Here

- use `cargo install bijux-cli` when you want the `bijux` runtime
- use `cargo install bijux-dag-cli` when you want the standalone DAG command
- use the [CLI handbook](https://bijux.io/bijux-core/bijux-cli/) when the
  question is about `bijux`
- use the [DAG handbook](https://bijux.io/bijux-core/bijux-dag/) when the
  question is about `bijux-dag`
- use the [repository handbook](https://bijux.io/bijux-core/bijux-core/) when
  the question crosses package boundaries, release policy, or ownership

## Package Families

<!-- bijux-core-package-map:generated:start -->
The public package families in this repository are:

| Package | Purpose | Links |
| --- | --- | --- |
| `bijux-cli` | Public Rust runtime for the `bijux` command surface, including routing, runtime behavior, and deterministic output contracts. | <a href="https://crates.io/crates/bijux-cli"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-cli?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-cli"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://pypi.org/project/bijux-cli/"><img alt="PyPI" src="https://img.shields.io/pypi/v/bijux-cli?label=PyPI&logo=pypi" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/"><img alt="Docs" src="https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli"><img alt="GHCR" src="https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
| `bijux-dag-artifacts` | Artifact identity, storage layout, retention, integrity, and lineage helpers for retained DAG run evidence. | <a href="https://crates.io/crates/bijux-dag-artifacts"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-dag-artifacts?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-dag-artifacts"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-artifacts/"><img alt="Docs" src="https://img.shields.io/badge/docs-artifacts-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-artifacts"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
| `bijux-dag-core` | Deterministic graph kernel for parsing, validation, canonicalization, planning, and semantic identity. | <a href="https://crates.io/crates/bijux-dag-core"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-dag-core?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-dag-core"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-core/"><img alt="Docs" src="https://img.shields.io/badge/docs-core-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-core"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
| `bijux-dag-runtime` | Execution engine and replay policy layer for DAG runs, cache decisions, and retained runtime diagnostics. | <a href="https://crates.io/crates/bijux-dag-runtime"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-dag-runtime?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-dag-runtime"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-runtime/"><img alt="Docs" src="https://img.shields.io/badge/docs-runtime-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-runtime"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
| `bijux-dag-app` | Application orchestration and response-shaping layer that turns DAG runtime behavior into user-facing workflows. | <a href="https://crates.io/crates/bijux-dag-app"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-dag-app?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-dag-app"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-app/"><img alt="Docs" src="https://img.shields.io/badge/docs-app-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-app"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
| `bijux-dag-cli` | Installable `bijux-dag` command package for validating, running, replaying, and inspecting DAG workflows. | <a href="https://crates.io/crates/bijux-dag-cli"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-dag-cli?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-dag-cli"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/"><img alt="Docs" src="https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag"><img alt="GHCR" src="https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-cli"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
<!-- bijux-core-package-map:generated:end -->

Within those families, the workspace currently contains:

- public `bijux` crates: `bijux-cli`
- public `bijux-dag` crates: `bijux-dag-core`, `bijux-dag-artifacts`,
  `bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-cli`
- repository-internal support crates: `bijux-cli-python`, `bijux-dag-testkit`,
  `bijux-dev`

The canonical publication boundary lives in
[`docs/bijux-core/foundation/package-boundary.md`](docs/bijux-core/foundation/package-boundary.md)
and `contracts/foundation/workspace_package_boundary.v1.json`.

## Public Rust Import Lanes

The public DAG crates publish an intentional Rust docs surface.

- browse `bijux_dag_core::stable` for the long-lived graph authoring,
  validation, and planning lane
- browse `bijux_dag_artifacts::stable` for the long-lived artifact identity,
  persistence, and integrity lane
- browse `bijux_dag_runtime::stable` for the long-lived execution, replay, and
  scheduling lane
- browse `bijux_dag_app::stable` for the long-lived command-orchestration and
  response-shaping lane
- use each crate's `prelude` module for common workflows
- use focused crate-root imports only when you already know the exact item you
  need
- broad compatibility re-exports remain callable for Rust consumers, but they
  stay hidden from the primary docs.rs lane so the published API surface reads
  like a product boundary instead of an internal module dump

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

## Workspace Pillars

- [`crates/`](https://github.com/bijux/bijux-core/tree/main/crates) contains
  the Rust package boundaries for public products and internal support crates.
- [`docs/`](https://github.com/bijux/bijux-core/tree/main/docs) contains the
  published handbooks for repository, CLI, DAG, and maintainer surfaces.
- [`contracts/`](https://github.com/bijux/bijux-core/tree/main/contracts)
  contains machine-checkable compatibility, schema, and release-boundary
  contracts.
- [`makes/`](https://github.com/bijux/bijux-core/tree/main/makes) contains the
  repository make modules used by local workflows and CI.
- [`evidence/dag/`](https://github.com/bijux/bijux-core/tree/main/evidence/dag)
  contains governed DAG fixtures, scenarios, and proof material used across
  docs, tests, and release verification.

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

Inspect the product command surfaces:

```bash
cargo run -p bijux-cli --bin bijux -- --help
cargo run -p bijux-dag-cli --bin bijux-dag -- --help
```

Run the repository-backed DAG demonstration:

```bash
make dag-demo
```

`make dag-demo` validates, executes, caches, replays, and strictly verifies a
real file-processing graph. It writes all retained output under
`artifacts/dag-demo/`. The [first-run tutorial](docs/bijux-dag/operations/first-run-tutorial.md)
explains each result and the [run evidence reference](docs/bijux-dag/interfaces/run-evidence-layout.md)
defines the retained files.

Use [Runnable Examples](docs/bijux-dag/interfaces/runnable-examples.md) for other
repository-backed workflows, [Security And Isolation Truth](docs/bijux-dag/operations/security-isolation-truth.md)
for the actual host boundary, [v0.4.0 Release Notes](docs/bijux-dag/operations/v0-4-0-release-notes.md)
for the shipped release, and the [Bijux Dag Roadmap](docs/tracking/bijux-dag-roadmap.md)
only for non-binding future direction.
The [Reproducibility Model](docs/bijux-dag/interfaces/reproducibility-model.md)
defines graph, plan, execution, cache, and replay identity.

## Documentation

| Handbook | Use it for |
| --- | --- |
| [Repository handbook](https://bijux.io/bijux-core/bijux-core/) | workspace scope, package ownership, release policy, shared architecture, and repository operations |
| [CLI handbook](https://bijux.io/bijux-core/bijux-cli/) | the `bijux` runtime, app and plugin routing, config behavior, diagnostics, and Python packaging |
| [DAG handbook](https://bijux.io/bijux-core/bijux-dag/) | DAG validation, planning, execution, replay, artifacts, compatibility, and operator workflows |
| [Maintainer handbook](https://bijux.io/bijux-core/bijux-dev/) | repository gates, release verification, docs operations, governance, and evidence collection |

The website is the reader-facing guide. Machine-enforced specifications and
generated evidence remain in the repository, but are not substitutes for
product documentation. The
[documentation system](docs/bijux-core/foundation/documentation-system.md)
defines those boundaries.

## Maintainer Workflows

```bash
make help
make docs-check
make dag-test
make dag-contracts
```

This repository keeps public product code, contracts, documentation, and
release proof together so drift becomes a reviewable code change rather than an
undocumented side channel.

## License

Apache-2.0 ([`LICENSE`](https://github.com/bijux/bijux-core/blob/main/LICENSE)).
