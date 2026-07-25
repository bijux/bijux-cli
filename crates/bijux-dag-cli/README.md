# bijux-dag-cli

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-dag-cli?label=crates.io&logo=rust)](https://crates.io/crates/bijux-dag-cli)
[![Rust docs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/) [![bijux-dag-cli docs](https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/)
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

- [Executable Examples](https://bijux.io/bijux-core/bijux-dag/interfaces/runnable-examples/)
  maps the repository-backed hello, file-processing, cache, replay, failure,
  branch, and container proofs to their expected outputs.
- [File Processing Workflow](https://bijux.io/bijux-core/bijux-dag/operations/file-processing-workflow/)
  demonstrates a host-shell artifact workflow with replay and promotion.
- [Cache Behavior Workflow](https://bijux.io/bijux-core/bijux-dag/operations/cache-behavior-workflow/)
  demonstrates stable cache verification, explicit-path cache-miss
  explanation, selective invalidation, and corruption refusal.
- [Reproducibility Model](https://bijux.io/bijux-core/bijux-dag/interfaces/reproducibility-model/)
  explains the retained identity surfaces behind cache verification, replay,
  export bundles, and artifact comparison.
- [Data Pipeline Workflow](https://bijux.io/bijux-core/bijux-dag/operations/data-pipeline-workflow/)
  demonstrates retained-run comparison and changed-input attribution.
- [Branching Bulletin Workflow](https://bijux.io/bijux-core/bijux-dag/operations/branching-bulletin-workflow/)
  demonstrates retained branch decisions, skipped lanes, join-trigger evidence,
  and replay stability.
- [Compliance-Gated Bulletin Workflow](https://bijux.io/bijux-core/bijux-dag/operations/compliance-gated-bulletin-workflow/)
  demonstrates transient retry evidence, focused replay repair, and strict
  verification after recovery.
- [Container Packaging Workflow](https://bijux.io/bijux-core/bijux-dag/operations/container-packaging-workflow/)
  demonstrates mounted container inputs, retained outputs, and recorded image
  identity.

Internal schedule and backfill guides remain available in the DAG handbook for
deliberate maintainer work with `BIJUX_DAG_ENABLE_INTERNAL=1`, but they are not
front-door examples for the public `bijux-dag` package.

## Internal Documentation

- [`ARCHITECTURE.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-cli/docs/ARCHITECTURE.md): thin entrypoint flow, dependency
  boundary, ownership, panic containment, and change decisions.
- [`COMPLETIONS_AND_COMMAND_SURFACE.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-cli/docs/COMPLETIONS_AND_COMMAND_SURFACE.md):
  app-owned command authority, supported shells, compatibility, and references.
- [`CONTRACTS.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-cli/docs/CONTRACTS.md): process ownership, thinness,
  compatibility, failure, and verification contracts.
- [`INSTALLED_BINARY_CONTRACT.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-cli/docs/INSTALLED_BINARY_CONTRACT.md):
  executable identity, argv, streams, status, isolation, and release evidence.
- [`PROCESS_AND_EXIT.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-cli/docs/PROCESS_AND_EXIT.md): parsing, dispatch status,
  streams, exit classes, signals, and process testing.
- [`TESTING_AND_RELEASE.md`](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-cli/docs/TESTING_AND_RELEASE.md): test layers,
  isolation, release checks, and failure ownership.

## Related links

- [Crate contracts](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-cli/docs/CONTRACTS.md)
- [Crate changelog](https://github.com/bijux/bijux-core/blob/main/crates/bijux-dag-cli/CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/)
