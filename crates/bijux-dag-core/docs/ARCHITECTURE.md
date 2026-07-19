# `bijux-dag-core` Architecture

`bijux-dag-core` is the pure graph kernel for Bijux DAG. It converts authored
graph data into validated, canonical, planner-ready structures without reading
runtime state or performing external effects.

## Processing Pipeline

```text
serialized graph
      |
      v
strict parsing -> input/reference resolution -> validation
      |                                         |
      v                                         v
canonical graph ------------------------> planner lowering
      |
      v
graph and node identity
```

Each operation receives all required data from the caller. Runtime scheduling,
artifact layout, process execution, clocks, and environment discovery happen
in downstream crates.

## Source Boundaries

| Area | Responsibility |
| --- | --- |
| `graph` | model, inputs, nodes, edges, resources, composition, expansion, topology |
| `pipeline` | strict parsing, reference resolution, and validation entrypoints |
| `analysis` | identity inputs, effects, trigger rules, and semantic analysis |
| `planner` | deterministic lowering into `ExecutionPlan` |
| `build` | builders, compile wrappers, lint, dry-run, and simulation helpers |
| `contracts` | typed invariant and compatibility evaluations |
| `lib.rs` | stable, prelude, compatibility, and experimental exports |

Domain logic belongs in these modules. The crate root curates access; it must
not become a second implementation of parsing, validation, or planning.

## Purity Rules

Core may deserialize, normalize Unicode, use deterministic collections, hash
canonical bytes, validate graph semantics, and lower a graph into a plan. It
must not access files, environment variables, processes, terminals, network,
wall-clock time, random execution identity, artifacts, or cache state.

The `Graph` model represents requested semantics, not observations about a
specific machine or run.

## Dependency Direction

`bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-testkit`, and `bijux-dev` may
depend on this crate. Core must not import them. `bijux-dag-artifacts` is a
sibling authority for retained evidence; graph validity and identity cannot
depend on a run directory.

## Extension Decisions

- Add authored graph data to the graph domain and define serialization rules.
- Add deterministic diagnostics for invalid input identified before planning.
- Add planner fields only when execution requires the lowered fact.
- Pass runtime-only values to runtime rather than adding them to `Graph`.
- Keep compatibility-sensitive entrypoints in `stable`.
- Put research contracts behind `experimental-public-api`.

## Verification

Deterministic, round-trip, canonicalization, validation, and planner contracts
protect these boundaries. Broad kernel changes should run:

```bash
cargo test --locked -p bijux-dag-core
```
