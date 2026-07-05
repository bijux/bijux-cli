---
title: DAG Handbook
audience: mixed
type: index
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-04
---

# DAG Handbook

`bijux-dag` is the graph execution and evidence subsystem in `bijux-core`. It
owns deterministic DAG semantics, run and artifact identity, replay
classification, and diff classification. The `v0.4.0` release makes five DAG
Rust crates public for the first time: `bijux-dag-core`,
`bijux-dag-artifacts`, `bijux-dag-runtime`, `bijux-dag-app`, and
`bijux-dag-cli`. `bijux-dag-testkit` remains repository-internal test support.

Runtime identity in manifests, provenance, replay, and cache fingerprints is
resolved from build metadata. Changing the shell directory around the compiled
binary is not supposed to rewrite DAG evidence identity.

The public operator contract is the visible `bijux-dag --help` surface. That
root help stays intentionally concise for `v0.4.0`. Hidden experimental,
simulation, and maintainer namespaces remain in the repository for internal
coverage and evidence work, but they are not presented as stable `v0.4.0`
operator APIs.

## v0.4.0 Surface Truth Table

| Class | `v0.4.0` meaning | Representative surfaces |
| --- | --- | --- |
| stable | supported visible `bijux-dag --help` surface for local DAG authoring, execution, replay, and evidence inspection | `validate`, `plan`, `run`, `replay`, `runs ...`, `artifact`, `artifact-inspect`, `diff`, `explain`, `verify`, `doctor`, `cache`, `version`, `commands`, `completions` |
| experimental | callable by explicit path and repository-tested, but outside the stable operator compatibility lane | `init`, `canonicalize`, `graph`, `graph-lint`, `fingerprint`, `hash`, `status`, `node`, `trace-artifact`, `why-rerun`, `why-cache-missed`, `export`, `import`, `migrate`, `adapters`, `config`, `policy`, `fsck`, `prove`, `proof-summary` |
| simulated | modeled platform and control-plane namespaces, not production backends or services | `control-plane`, `state-store`, `dataset`, `enterprise`, `fleet`, `federation`, `incident`, `lab` |
| internal | maintainer-only and contract-only routes outside the public operator boundary | `security`, `durability`, `performance`, `release`, `runtime`, `schedule`, `version-inspect`, `capabilities`, `semantic-portability`, `equivalence-proof` |
| future | not a `v0.4.0` product promise | kubernetes execution, slurm or hpc execution, remote workers, public enterprise or federation APIs, full scheduler service |

For the canonical source, use
[Release Boundary](foundation/release-boundary.md).

Use this handbook when the question is about graph truth, execution policy,
replay outcomes, artifact behavior, or how the DAG crates divide ownership.

<div class="bijux-quicklinks">
<a class="md-button md-button--primary" href="packages/bijux-dag-core.md">Open the kernel package</a>
<a class="md-button" href="packages/bijux-dag-runtime.md">Open the runtime package</a>
<a class="md-button" href="packages/bijux-dag-app.md">Open the app package</a>
</div>

## Package Map

```mermaid
flowchart LR
    handbook["DAG handbook"] --> core["core"]
    handbook --> runtime["runtime"]
    handbook --> app["app"]
    handbook --> artifacts["artifacts"]
```

## Package Destinations

- [`bijux-dag-core`](packages/bijux-dag-core.md) owns graph truth and planner lowering
- [`bijux-dag-runtime`](packages/bijux-dag-runtime.md) owns execution policy, replay, and diagnostics
- [`bijux-dag-app`](packages/bijux-dag-app.md) owns command orchestration and response shaping
- [`bijux-dag-cli`](packages/bijux-dag-cli.md) owns the thin executable wrapper
- [`bijux-dag-artifacts`](packages/bijux-dag-artifacts.md) owns artifact identity, integrity, and lifecycle helpers
- [`bijux-dag-testkit`](packages/bijux-dag-testkit.md) owns shared deterministic fixtures for repository tests and maintainer suites

## Code Anchors

- `crates/bijux-dag-cli/src/main.rs`
- `crates/bijux-dag-app/src/`
- `crates/bijux-dag-core/src/`
- `crates/bijux-dag-runtime/src/`
- `crates/bijux-dag-artifacts/src/`

## Main Paths

- [Foundation](foundation/index.md)
- [Architecture](architecture/index.md)
- [Interfaces](interfaces/index.md)
- [Operations](operations/index.md)
- [Quality](quality/index.md)

## Related Handbooks

- [Repository Handbook](../bijux-core/index.md)
- [CLI Handbook](../bijux-cli/index.md)
- [Maintainer Handbook](../bijux-dev/index.md)

## Contract Anchors

- [Planner Contract](../spec/PLANNER_CONTRACT.md)
- [Replay Contract](../spec/REPLAY_CONTRACT.md)
