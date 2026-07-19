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

`bijux-core` ships two public commands and the crates that implement them.

| Product | Install | Use it for | Primary authority |
| --- | --- | --- | --- |
| `bijux` | `cargo install bijux-cli` or `python -m pip install bijux-cli` | mounted apps, plugins, layered configuration, diagnostics, history, memory, and REPL workflows | [CLI handbook](https://bijux.io/bijux-core/bijux-cli/) |
| `bijux-dag` | `cargo install bijux-dag-cli` | graph validation, planning, execution, retained evidence, cache explanation, replay, comparison, and verification | [DAG handbook](https://bijux.io/bijux-core/bijux-dag/) |

`bijux-dev`, the executable specifications, and governed reports are repository
maintenance surfaces. They are not additional end-user products.

The current workspace release line is `0.4.0`.

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles.

### `bijux-dag` v0.4.0 Surface Truth Table

The machine-readable release authority is
[`contracts/foundation/dag_release_truth_table.v1.json`](contracts/foundation/dag_release_truth_table.v1.json).

| Class | Current boundary |
| --- | --- |
| stable | visible `bijux-dag --help` commands plus bounded local, container, shared-filesystem SLURM, and Kubernetes Job execution |
| experimental | callable explicit-path helpers outside the compatibility promise |
| simulated | modeled namespaces inventoried with `commands --lane simulated` and gated by `BIJUX_DAG_ENABLE_SIMULATED=1` |
| internal | maintainer routes inventoried with `commands --lane internal` and gated by `BIJUX_DAG_ENABLE_INTERNAL=1` |
| unreleased | generic HPC, public remote workers, scheduler services, and public enterprise or federation APIs |

Use the [Release Boundary](docs/bijux-dag/foundation/release-boundary.md) for
operator decisions, the [v0.4.0 Release Notes](docs/bijux-dag/operations/v0-4-0-release-notes.md)
for the shipped release, and the [Bijux Dag Roadmap](docs/bijux-dag/roadmap.md)
only for non-binding future direction.

## Install And Inspect

```bash
cargo install bijux-cli
cargo install bijux-dag-cli

bijux --help
bijux-dag --help
```

The PyPI package installs `bijux`; it does not install `bijux-dag`. See the
[Python package boundary](crates/bijux-cli-python/README.md) before using the
Python DAG client.

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

The canonical package publication boundary lives in
[`docs/bijux-core/foundation/package-boundary.md`](docs/bijux-core/foundation/package-boundary.md)
and `contracts/foundation/workspace_package_boundary.v1.json`.

## Develop The Repository

Local builds, CI, and release jobs all use the pinned Rust `1.86.0` toolchain
declared in `rust-toolchain.toml`.

```bash
make bootstrap
make doctor
make test
```

`make test` runs the fast Rust lane and Python tests not marked `nightly`.
It does not run governed slow Rust tests, ignored Rust tests, Python nightly
tests, documentation checks, or lint. Use
[Testing And Validation](docs/bijux-core/operations/testing-and-validation.md)
to select a broader lane without overstating what passed.

```bash
make dag-demo
```

`make dag-demo` writes retained proof under `artifacts/dag-demo/`. The
[First-Run Tutorial](docs/bijux-dag/operations/first-run-tutorial.md) explains
how to inspect and verify it.

## Documentation Authority

| Question | Authority |
| --- | --- |
| how do I run a checked-in workflow? | [Executable Examples](docs/bijux-dag/interfaces/runnable-examples.md) |
| which retained files prove a run? | [Run Evidence Layout](docs/bijux-dag/interfaces/run-evidence-layout.md) |
| what makes two runs equivalent? | [Reproducibility Model](docs/bijux-dag/interfaces/reproducibility-model.md) |
| what isolation is actually enforced? | [Execution Security And Isolation](docs/bijux-dag/operations/security-isolation-truth.md) |
| how are repository docs divided? | [Documentation System](docs/bijux-core/foundation/documentation-system.md) |
| how do maintainers validate changes? | [Maintainer handbook](https://bijux.io/bijux-core/bijux-dev/) |

`docs/spec/` contains executable contracts and `docs/reports/` contains
governed evidence. Neither directory is published as a reader handbook, and
local logs or generated sites belong under `artifacts/`.

## License

Apache-2.0 ([`LICENSE`](https://github.com/bijux/bijux-core/blob/main/LICENSE)).
