---
title: Release Boundary
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Release Boundary

`bijux-dag` is only trustworthy if a reader can tell which surfaces are
supported in `v0.4.0`, which ones are still experimental, which ones are
simulation-only, and which claims remain unreleased.

The contract source for this page is
`contracts/foundation/dag_release_truth_table.v1.json`.

This page governs operator-surface release status. For public/private crate
publication status, use
[Package Boundary](../../bijux-core/foundation/package-boundary.md) and
`contracts/foundation/workspace_package_boundary.v1.json`.
For post-`v0.4.0` direction that is not yet a release promise, use the
[Future Direction](future-direction.md).

## v0.4.0 Surface Truth Table

| Class | `v0.4.0` meaning | Representative surfaces |
| --- | --- | --- |
| stable | supported visible `bijux-dag --help` surface for local DAG authoring, execution, replay, and evidence inspection | `validate`, `plan`, `run`, `replay`, `runs ...`, `artifact`, `artifact-inspect`, `diff`, `explain`, `verify`, `doctor`, `cache`, `version`, `commands`, `completions` |
| experimental | callable by explicit path and repository-tested, but outside the stable operator compatibility lane | `init`, `canonicalize`, `graph`, `graph-lint`, `fingerprint`, `hash`, `status`, `node`, `trace-artifact`, `why-rerun`, `why-cache-missed`, `export`, `import`, `migrate`, `adapters`, `config`, `policy`, `fsck`, `prove`, `proof-summary` |
| simulated | modeled platform and control-plane namespaces that require `BIJUX_DAG_ENABLE_SIMULATED=1`, not production backends or services | `control-plane`, `state-store`, `dataset`, `enterprise`, `fleet`, `governance`, `federation`, `incident`, `lab` |
| internal | maintainer-only and contract-only routes that require `BIJUX_DAG_ENABLE_INTERNAL=1` and stay outside the public operator boundary | `security`, `durability`, `performance`, `release`, `runtime`, `schedule`, `version-inspect`, `capabilities`, `semantic-portability`, `equivalence-proof` |
| unreleased | not a `v0.4.0` product promise | generic hpc execution beyond the shared-filesystem slurm lane, public remote workers, public enterprise or federation APIs, full scheduler service |

## Stable Capabilities

The stable `v0.4.0` release contract covers:

- local DAG validation
- local DAG execution
- Kubernetes Job submission through `run --backend kubernetes` for container
  nodes, including request and limit mapping, active deadline mapping, pod
  phase mapping, shared-volume workspace mounting, and retained batch evidence
- shared-filesystem SLURM submission through `run --backend slurm`, including
  `sbatch` submission, `sacct` polling, and retained batch evidence
- run and artifact evidence inspection
- replay and diff classification
- cache verification and maintenance
- machine-readable CLI JSON output

Use `bijux-dag commands` to inspect the stable operator surface itself.
Inventory gated routes only by deliberate lane:
`bijux-dag commands --lane experimental`,
`bijux-dag commands --lane simulated`, or
`bijux-dag commands --lane internal`.

## Reading Rules

- build operator procedures on the stable row only
- use `bijux-dag commands --lane experimental` when you intentionally need
  repository-tested but non-stable operator helpers
- use `bijux-dag commands --lane simulated` or `bijux-dag commands --lane internal`
  only for deliberate modeled or maintainer workflows
- set `BIJUX_DAG_ENABLE_SIMULATED=1` or `BIJUX_DAG_ENABLE_INTERNAL=1` only for
  deliberate maintainer or contract workflows
- do not treat simulated namespaces as production backends
- do not describe generic scheduler services or public remote workers as already shipped

## Code Anchors

- `contracts/foundation/dag_release_truth_table.v1.json`
- `crates/bijux-dag-app/src/commands/mod.rs`
- `crates/bijux-dag-app/src/routes/command_routes.rs`
- `crates/bijux-dag-cli/src/main.rs`

## Next Reads

- [CLI Surface](../interfaces/cli-surface.md)
- [Package Boundary](../../bijux-core/foundation/package-boundary.md)
- [Future Direction](future-direction.md)
- [Scope and Boundaries](scope-and-boundaries.md)
- [Known Limitations](../quality/known-limitations.md)
