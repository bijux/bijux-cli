---
title: Entrypoints and Examples
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Entrypoints and Examples

Use this page to choose a trustworthy starting point. It deliberately does not
copy command sequences from the pages that execute and verify them.

`bijux-dag` v0.4.1 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. The
[Release Boundary](../foundation/release-boundary.md) classifies each command
lane, while `contracts/foundation/dag_release_truth_table.v1.json` is the
machine-readable authority.

## Choose By Intent

| Intent | Start here | Evidence you should obtain |
| --- | --- | --- |
| prove one complete local run | [First-Run Tutorial](../operations/first-run-tutorial.md) | validated graph, retained artifacts, warm cache reuse, focused replay, strict verification |
| choose a checked-in example | [Executable Examples](runnable-examples.md) | exact graph or guide, commands, and expected outputs |
| understand the stable command surface | [CLI Surface](cli-surface.md) | command purpose, inputs, output shape, and lane classification |
| author a graph | [Authoring Guide](authoring-guide.md) | valid graph shape and explicit rejection cases |
| inspect retained run evidence | [Run Evidence Layout](run-evidence-layout.md) | authoritative file locations and ownership |
| investigate cache behavior | [Cache Behavior Workflow](../operations/cache-behavior-workflow.md) | warm reuse, invalidation, corruption refusal, and miss explanation |
| compare changed inputs and affected nodes | [Data Pipeline Workflow](../operations/data-pipeline-workflow.md) | retained-run comparison and changed-input attribution |
| test conditional execution | [Branching Bulletin Workflow](../operations/branching-bulletin-workflow.md) | selected lane, skipped lane, join behavior, and replay stability |
| test retry and focused repair | [Compliance-Gated Bulletin Workflow](../operations/compliance-gated-bulletin-workflow.md) | retry record, root failure, propagated fallout, and repaired run |
| run a container-backed node | [Container Packaging Workflow](../operations/container-packaging-workflow.md) | mounted inputs, retained outputs, and engine identity |

## Proof Boundaries

The product claims map to inspectable evidence:

- `validate` proves that a graph satisfies the accepted contract before work
  starts.
- a retained run directory records node execution and artifact identity.
- `artifact registry` and `artifact-inspect` expose promoted output evidence.
- replay and strict verification test whether retained evidence can support the
  claimed reproduction.
- semantic comparison attributes relevant changes between retained runs.

The [Replay Contract](../../spec/REPLAY_CONTRACT.md) governs replay identity.
The executable catalog is the authority for copyable repository commands; the
workflow guides own interpretation and recovery.

## Command Lane Rule

Examples intended for users must stay on the stable visible CLI surface.
Repository-tested experimental routes may appear only when the page labels
them as experimental and shows the explicit invocation path. Internal and
simulated routes are evidence for maintainers, not implied public support.

Check the generated command reference when command syntax matters. Check the
release boundary before putting any command into automation.

## Rust Library Entrypoint

The core parser is available directly when an application owns orchestration
and only needs graph validation:

```rust
use bijux_dag_core::parse_graph_strict;

let graph = parse_graph_strict("{\"spec\":\"bijux-dag/v0.1\",\"nodes\":[],\"edges\":[]}")?;
println!("spec={}", graph.spec);
```

This example demonstrates parsing only. Execution, retained evidence, replay,
and artifact policy belong to the runtime and application layers; do not infer
those guarantees from a successful parse.

## Code Anchors

- `crates/bijux-dag-cli/src/main.rs`
- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dag-core/src/lib.rs`
- `crates/bijux-dag-runtime/src/lib.rs`

## Next Reads

- [Executable Examples](runnable-examples.md)
- [First-Run Tutorial](../operations/first-run-tutorial.md)
- [Operator Workflows](operator-workflows.md)
- [Generated CLI Reference](generated-cli-reference.md)
- [Gated Command Inventory](gated-command-inventory.md)
- [Known Limitations](../quality/known-limitations.md)
