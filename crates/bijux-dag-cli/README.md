# bijux-dag-cli

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-cli?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-cli)
[![Rust docs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-dag-cli](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag-artifacts](https://img.shields.io/crates/v/bijux-dag-artifacts?label=artifacts&logo=rust)](https://crates.io/crates/bijux-dag-artifacts) [![bijux-dag-core](https://img.shields.io/crates/v/bijux-dag-core?label=core&logo=rust)](https://crates.io/crates/bijux-dag-core) [![bijux-dag-runtime](https://img.shields.io/crates/v/bijux-dag-runtime?label=runtime&logo=rust)](https://crates.io/crates/bijux-dag-runtime) [![bijux-dag-app](https://img.shields.io/crates/v/bijux-dag-app?label=app&logo=rust)](https://crates.io/crates/bijux-dag-app) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-dag-cli](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-dag-cli docs](https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/) [![bijux-dag-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag-artifacts docs.rs](https://img.shields.io/badge/rust--docs-artifacts-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-artifacts) [![bijux-dag-core docs.rs](https://img.shields.io/badge/rust--docs-core-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-core) [![bijux-dag-runtime docs.rs](https://img.shields.io/badge/rust--docs-runtime-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-runtime) [![bijux-dag-app docs.rs](https://img.shields.io/badge/rust--docs-app-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-app)
<!-- bijux-core-badges:generated:end -->

`bijux-dag-cli` installs the `bijux-dag` executable. It is the public command
package for the DAG product.

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. This crate is the
installed command surface for that promise.

Install it when you want the standalone DAG command surface:

```bash
cargo install bijux-dag-cli
bijux-dag --help
```

For most users, this is the package to start with. It gives you the supported
local DAG workflow surface without asking you to assemble the lower-level DAG
crates yourself.

## Release Status

- public crate on the `v0.4.0` DAG release line
- installs the stable operator-facing `bijux-dag` binary
- does not promote experimental, simulated, or internal namespaces into the
  default public contract

## Stable Operator Boundary

The supported release boundary is the visible `bijux-dag --help` surface:
`validate`, `plan`, `run`, `replay`, `runs`, `artifact`, `artifact-inspect`,
`diff`, `explain`, `verify`, `doctor`, `cache`, `version`, `commands`, and
`completions`.

Experimental routes remain available by explicit path for repository-owned
workflows, and `bijux-dag commands --lane experimental` is the deliberate
inventory surface for that lane. Simulated and maintainer namespaces require
explicit lane inventory through `bijux-dag commands --lane simulated` or
`bijux-dag commands --lane internal`, plus execution opt-in through
`BIJUX_DAG_ENABLE_SIMULATED=1` or `BIJUX_DAG_ENABLE_INTERNAL=1`.

Container-backed nodes are part of the stable local execution surface. When a
supported engine such as Docker is available on `PATH`, `bijux-dag run`
executes the container step, retains stdout and stderr, and records engine and
image identity in the retained trace. When the engine is missing, the run
fails as an infrastructure error rather than silently degrading to another
execution mode.

Branch-backed workflows are also part of the stable local execution surface.
When a DAG uses a branch node, retained runs record the selected decision, the
skipped lane, and the join-node trigger evaluation so operators can inspect the
execution path directly from run evidence.

## What It Provides

- the `bijux-dag` binary entrypoint
- thin startup wiring, process initialization, and exit mapping
- delegation into `bijux-dag-app` for actual command behavior
- shell completion generation for the installed executable
- lane-scoped command discovery for stable, experimental, simulated, and
  maintainer-only route inventories

## What It Does Not Own

- graph semantics
- runtime execution logic
- artifact persistence rules

If the question is about route behavior rather than process startup, the next
place to read is usually `bijux-dag-app`.

## Representative Workflows

- [Executable Examples](../../docs/bijux-dag/interfaces/runnable-examples.md)
  maps the repository-backed hello, file-processing, cache, replay, failure,
  branch, and container proofs to their expected outputs.
- [File Processing Workflow](../../docs/bijux-dag/operations/file-processing-workflow.md)
  demonstrates a host-shell artifact workflow with replay and promotion.
- [Cache Behavior Workflow](../../docs/bijux-dag/operations/cache-behavior-workflow.md)
  demonstrates stable cache verification, explicit-path cache-miss
  explanation, selective invalidation, and corruption refusal.
- [Reproducibility Model](../../docs/bijux-dag/interfaces/reproducibility-model.md)
  explains the retained identity surfaces behind cache verification, replay,
  export bundles, and artifact comparison.
- [Data Pipeline Workflow](../../docs/bijux-dag/operations/data-pipeline-workflow.md)
  demonstrates retained-run comparison and changed-input attribution.
- [Branching Bulletin Workflow](../../docs/bijux-dag/operations/branching-bulletin-workflow.md)
  demonstrates retained branch decisions, skipped lanes, join-trigger evidence,
  and replay stability.
- [Compliance-Gated Bulletin Workflow](../../docs/bijux-dag/operations/compliance-gated-bulletin-workflow.md)
  demonstrates transient retry evidence, focused replay repair, and strict
  verification after recovery.
- [Container Packaging Workflow](../../docs/bijux-dag/operations/container-packaging-workflow.md)
  demonstrates mounted container inputs, retained outputs, and recorded image
  identity.

Internal schedule and backfill guides remain available in the DAG handbook for
deliberate maintainer work with `BIJUX_DAG_ENABLE_INTERNAL=1`, but they are not
front-door examples for the public `bijux-dag` package.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/)
