---
title: DAG Handbook
audience: mixed
type: index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# DAG Handbook

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles (see [Replay Contract](../spec/REPLAY_CONTRACT.md)).

Inside `bijux-core`, that promise covers graph validation, execution planning,
local run orchestration, replay, artifact identity, evidence inspection, cache
reasoning, and changed-run comparison.

Use this handbook when the question is about DAG behavior itself: what gets
validated, what gets executed, what evidence is written, what is stable today,
and which crate owns the answer once the route is clear.

## v0.4.0 Surface Truth Table

The supported operator boundary is the visible `bijux-dag --help` surface for
local DAG work:

- `validate`, `plan`, `run`, `replay`
- `runs`, `artifact`, `artifact-inspect`, `diff`, `explain`
- `verify`, `doctor`, `cache`, `version`, `commands`, `completions`

That stable lane is intentionally local-first. Repository-owned experimental,
simulated, and maintainer-only routes still exist, but they are deliberate
opt-in lanes rather than the default product story.

Use [Release Boundary](foundation/release-boundary.md) for the exact lane
classification, [Generated CLI Reference](interfaces/generated-cli-reference.md)
for the stable command surface generated from the live binary, and
[Gated Command Inventory](interfaces/reference/gated-command-inventory.md)
when you deliberately need the experimental, simulated, or internal route
inventory.

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="operations/guides/first-run-tutorial.md">Start with the first-run tutorial</a>
<a class="md-button" href="operations/v0-4-0-release-notes.md">Read the v0.4.0 release notes</a>
<a class="md-button" href="interfaces/examples/index.md">Open runnable examples</a>
<a class="md-button" href="packages/index.md">Open the package map</a>
</div>

## Start Here

| If you want to... | Open this page |
| --- | --- |
| get a working DAG run as fast as possible | [First-Run Tutorial](operations/guides/first-run-tutorial.md) |
| browse real workflows with expected outputs | [Runnable Examples](interfaces/examples/index.md) |
| check whether a command or backend is part of the shipped boundary | [Release Boundary](foundation/release-boundary.md) |
| understand retained run evidence on disk | [Run Evidence Layout](interfaces/reference/run-evidence-layout.md) |
| understand graph, plan, execution, cache, and replay identity | [Reproducibility Model](interfaces/reference/reproducibility-model.md) |
| find the owning crate before reading code | [DAG Packages](packages/index.md) |

## Product Proof Map

The public product sentence is only useful if a reader can trace each claim to
one concrete proof surface:

| Product claim | Where this handbook proves it |
| --- | --- |
| explicit graph contracts | [Graph Schema Reference](interfaces/reference/graph-schema.md) and [First-Run Tutorial](operations/guides/first-run-tutorial.md) |
| deterministic execution records | [Run Evidence Layout](interfaces/reference/run-evidence-layout.md) and [Operator Workflows](interfaces/operator-workflows.md) |
| verified artifacts | [Artifact Contracts](interfaces/artifact-contracts.md) and [First-Run Tutorial](operations/guides/first-run-tutorial.md) |
| cache explanation | [Cache Behavior Workflow](operations/guides/cache-behavior-workflow.md) and [CLI Surface](interfaces/cli-surface.md) |
| replayable run bundles | [Reproducibility Model](interfaces/reference/reproducibility-model.md), [Failure Recovery](operations/failure-recovery.md), and [Replay Contract](../spec/REPLAY_CONTRACT.md) |

## Packages In This Product

The current public DAG crate family is:

- `bijux-dag-core` for graph truth and planner inputs
- `bijux-dag-artifacts` for run evidence, integrity, and lifecycle helpers
- `bijux-dag-runtime` for execution policy, replay, cache, and diagnostics
- `bijux-dag-app` for command orchestration and response shaping
- `bijux-dag-cli` for the thin `bijux-dag` executable wrapper

`bijux-dag-testkit` remains repository-internal support for deterministic DAG
fixtures and shared assertions.

For the public-versus-private crate boundary behind that split, use
[`../bijux-core/foundation/package-boundary.md`](../bijux-core/foundation/package-boundary.md).

## Honest Boundary Notes

- `run --backend slurm` is part of the current release line for
  shared-filesystem environments where scheduled workers can reopen the
  retained run directory.
- `run --backend kubernetes` is part of the current release line for
  container-node execution through Kubernetes Jobs with shared persistent
  storage.
- Experimental routes remain callable by explicit path and are visible through
  `bijux-dag commands --lane experimental`.
- Simulated and maintainer namespaces require explicit opt-in through
  `BIJUX_DAG_ENABLE_SIMULATED=1` or `BIJUX_DAG_ENABLE_INTERNAL=1` together with
  deliberate lane inventory.
- If the next question sounds like a security claim rather than a workflow
  claim, route it to
  [Security And Isolation Truth](operations/reference/security-isolation-truth.md)
  before treating a flag or backend as an enforced boundary.

## Good First Reads

- [CLI Surface](interfaces/cli-surface.md) for the operator contract
- [Graph Schema Reference](interfaces/reference/graph-schema.md) for authoring
  truth
- [Cache Behavior Workflow](operations/guides/cache-behavior-workflow.md) for
  reuse, verification, and refusal behavior
- [Container Packaging Workflow](operations/guides/container-packaging-workflow.md)
  for container-backed execution
- [Branching Bulletin Workflow](operations/guides/branching-bulletin-workflow.md)
  for branch decisions, skipped lanes, and join behavior

## When To Leave This Handbook

- Move to the [Repository Handbook](../bijux-core/index.md) when the answer
  depends on publication rules, shared release policy, or cross-product
  ownership.
- Move to the [Maintainer Handbook](../bijux-dev/index.md) when the work is
  about governance suites, release proof, or repository gates.
- Move to the [Bijux Dag Roadmap](../tracking/bijux-dag-roadmap.md) only when
  the question is about future work rather than shipped `v0.4.0` behavior.
