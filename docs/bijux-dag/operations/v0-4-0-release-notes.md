---
title: v0.4.0 Release Notes
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# `bijux-dag` v0.4.0 Release Notes

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows with
explicit graph contracts, deterministic execution records, verified artifacts,
cache explanation, and replayable run bundles (see [Replay Contract](../../spec/REPLAY_CONTRACT.md)).

This release is intentionally serious and intentionally narrow. It ships a real
local product surface today. It does not claim to be a full distributed
scheduler, a public enterprise control plane, or a general Airflow replacement.

Use these notes when the release question is "what can I rely on now, what is
still non-stable, and what should I actually run to prove the answer?"

## Release Summary

- The first public DAG Rust crate family is now published for the `v0.4.0`
  release line: `bijux-dag-core`, `bijux-dag-artifacts`, `bijux-dag-runtime`,
  `bijux-dag-app`, and `bijux-dag-cli`.
- The stable operator contract is the visible `bijux-dag --help` surface for
  local validation, planning, execution, replay, inspection, cache work, and
  verification.
- Repository-owned experimental, simulated, and internal lanes remain
  documented because they exist, but they are not presented as shipped public
  scheduler, platform, or organization-control products.

## Stable Features

These are the `v0.4.0` capabilities you can treat as the honest public DAG
operator surface:

- local graph validation with explicit schema and compatibility checks
- graph planning and inspection before execution
- local run execution with isolated node work directories
- retained run manifests, traces, event timelines, and output indexes
- artifact registry, artifact inspection, lineage, and promotion
- deterministic graph and node fingerprints in retained evidence
- local cache reuse with explicit cache-hit reporting
- focused replay from retained upstream evidence
- strict run verification after execution or replay
- public crate surfaces for the DAG Rust package family

For the exact stable command inventory, use
[Release Boundary](../foundation/release-boundary.md) and
[Generated CLI Reference](../interfaces/generated-cli-reference.md).

## Experimental Features

These routes are real and repository-tested, but they are outside the stable
operator compatibility lane:

- explicit-path helper routes such as `init`, `status`, `export`, `import`,
  `migrate`, `prove`, `proof-summary`, `trace-artifact`, `why-rerun`, and
  `why-cache-missed`
- compatibility and migration helper routes that are useful for maintainer and
  advanced operator diagnostics
- hidden helper routes that can evolve within the `v0.4.x` line without
  changing the stable `bijux-dag --help` contract

Use `bijux-dag commands --lane experimental` only when you intentionally need
that repository-owned, non-stable inventory.

## Internal And Future Features

These surfaces exist in the repository, but they are not public `v0.4.0`
product promises:

- internal schedule and backfill lanes gated behind
  `BIJUX_DAG_ENABLE_INTERNAL=1`
- modeled platform and organization-control namespaces gated behind
  `BIJUX_DAG_ENABLE_SIMULATED=1`
- generic HPC beyond the shared-filesystem SLURM lane
- public remote workers, public scheduler services, and public federation or
  enterprise control planes

Two backend lanes are concrete in `v0.4.0`:

- shared-filesystem SLURM execution for batch submission on nodes that can
  reopen the retained run directory
- Kubernetes Job execution for container nodes with retained node inputs,
  outputs, and work directories mounted through a shared persistent volume
  claim

Those current backend lanes do not broaden the product claim into a general
distributed scheduler or platform-control service.

## Known Limitations

The release boundary tells you what is in scope. The limitations page tells you
what the shipped surface still does not guarantee.

Read [Known Limitations](../quality/known-limitations.md) alongside these
notes. The most important current limitations are:

- stable execution remains a single-controller local runtime
- shell policy denial is not a syscall sandbox
- container isolation claims depend on the selected runtime boundary
- internal schedule and backfill lanes are not stable scheduler APIs
- remote coordination and modeled distributed backends are not shipped product
  lanes
- hidden experimental routes remain callable but are not stable operator APIs

## Breaking Changes

`v0.4.0` is the first public release line for the DAG Rust crate family, so it
does not carry a prior public DAG crate contract that has to be preserved from
earlier crates.io versions.

The main compatibility line to watch in this release is command-surface
classification:

- the public DAG binary is `bijux-dag`
- the stable operator contract is the visible `bijux-dag --help` inventory
- repository-owned experimental, simulated, and internal routes are not part of
  that stable contract even when they are executable

If you previously depended on repository checkouts, hidden helper routes, or
internal schedule namespaces as if they were public APIs, treat that usage as
non-stable and reclassify it before rollout.

## Migration Notes

Operators adopting `v0.4.0` should align around the durable public forms:

- use `bijux-dag ...` for the public DAG binary
- use `bijux dag ...` only when you intentionally want root-managed app routing
- migrate deprecated alias wrappers such as `bijux-workflow ...` to
  `bijux-dag ...` or `bijux dag ...`
- build production automation on the visible `bijux-dag --help` surface, not
  on hidden explicit-path routes

For the routed-root compatibility details, use the CLI
[Migration Guide](../../bijux-cli/operations/migration-guide.md).

## Examples

The release is not meant to be taken on trust alone. These are the canonical
operator proofs:

- `make dag-demo` for the shortest honest end-to-end proof of local execution,
  retained artifacts, warm cache reuse, focused replay, and strict verification
- [First-Run Tutorial](first-run-tutorial.md) for the same proof path as
  a step-by-step walkthrough
- [File Processing Workflow](file-processing-workflow.md) for one real
  retained workflow family on the stable local operator surface
- [Data Pipeline Workflow](data-pipeline-workflow.md) for retained-run
  comparison and changed-input attribution
- [Cache Behavior Workflow](cache-behavior-workflow.md) for cache reuse,
  invalidation, and cache-miss explanation

## Validation Commands

These are the practical commands behind the release claim:

```bash
make dag-demo
make fmt
make lint
make release-validate-rs
```

For the exact release validation lane and what it proves, use
[Release Validation Suite](../../bijux-dev/operations/release-validation-suite.md)
and [Release and Versioning](release-and-versioning.md). The broad release
verification lane for this repository is `make release-validate-rs`.

## Evidence And Decision Sources

When you need the exact source behind a release claim, use these pages together:

- [Release Boundary](../foundation/release-boundary.md)
- [Known Limitations](../quality/known-limitations.md)
- [Risk Register](../quality/risk-register.md)
- [Generated CLI Reference](../interfaces/generated-cli-reference.md)
- [Future Direction](../foundation/future-direction.md) for the capability
  gates that govern what may come
  after the current release boundary
