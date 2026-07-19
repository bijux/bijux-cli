---
title: Current Implemented Capabilities
audience: mixed
type: explanation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-19
---

# Current Implemented Capabilities

This page identifies behavior backed by repository code and executable proof.
It prevents implementation presence, public support, and release stability from
being collapsed into one claim.

Implementation is necessary but not sufficient for a public promise. Use the
[DAG Support Matrix](../../bijux-dag/interfaces/support-matrix.md) and
[Release Boundary](../../bijux-dag/foundation/release-boundary.md) for the
operator commitment attached to each surface.

## Product Capabilities

| Capability | Implementation owner | Proof and limits |
| --- | --- | --- |
| strict graph parsing, validation, normalization, and planning | `bijux-dag-core` | graph schema, planner contracts, and core contract suites |
| local dependency scheduling, retries, cache use, and retained traces | `bijux-dag-runtime` | runtime semantics, execution-flow, cache, and state-machine contracts |
| shell, process, Python, HTTP, file-transform, and container adapters | runtime adapter registry | adapter contracts and per-kind execution suites; isolation varies by adapter |
| Kubernetes Job execution for container nodes | Kubernetes batch backend | requires the shared-volume and controller-root prerequisites in the support matrix |
| shared-filesystem SLURM execution | SLURM batch backend | requires `sbatch`, `sacct`, and a run directory visible to the worker |
| run manifests, artifact indexes, import/export, and integrity verification | `bijux-dag-artifacts` and application routes | run-directory and artifact lifecycle contracts |
| replay comparison, semantic diff, and bounded rerun from a node | runtime and application replay routes | [Replay Contract](../../spec/REPLAY_CONTRACT.md); missing or incompatible evidence is refused |
| run history, inspection, explanation, diagnostics, and repair proposals | `bijux-dag-app` | operator inspection contracts; proposals do not mutate authority until accepted |
| stable command discovery and machine-readable errors | CLI and application routes | generated CLI reference, command lanes, and error registry |

The table states that code and proof exist. It does not imply identical
security, recovery, portability, or compatibility guarantees across those
capabilities.

## Retained Evidence

Completed runs retain graph identity, node status, trace evidence, output
indexes, and integrity metadata under the governed run-directory contract.
Cache explanation and replay comparison depend on that retained evidence; they
cannot reconstruct proof that was never recorded.

Import, export, and replay claims remain versioned. Unknown schema or bundle
versions fail closed rather than being interpreted as current.

## Boundary Rule

Implemented does not mean public, stable, or broadly promised.

- Stable, experimental, simulated, internal, and unreleased command lanes have
  different compatibility commitments.
- A modeled contract can be executable in tests without being an operator
  product.
- An implemented backend still depends on external scheduler, storage, and
  host-security conditions.
- Maintainer commands prove repository properties; they are not application
  APIs.

The [Known Limitations](../../bijux-dag/quality/known-limitations.md) page owns
current constraints. [Package Boundary](package-boundary.md) and
[Ownership Model](ownership-model.md) explain crate and subsystem authority.

## Verification Entry Points

- `cargo run -p bijux-dag-cli --bin bijux-dag -- commands`
- `cargo run -p bijux-dag-cli --bin bijux-dag -- version`
- `cargo run -p bijux-dag-cli --bin bijux-dag -- doctor`
- `cargo run -p bijux-dev --bin bijux-dev-dag -- repo run`

This page excludes unshipped direction and speculative platform expansion.
The release boundary classifies modeled and unreleased surfaces without
presenting them as current capability.
