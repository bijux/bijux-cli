# `bijux-dag-core` Contracts

`bijux-dag-core` is the deterministic graph kernel. It defines what a DAG
means before runtime policy, filesystem state, clocks, or external processes
can influence execution.

## Owned Surface

The crate owns:

- graph, node, edge, input, resource, and metadata models;
- strict parsing and schema-aligned validation;
- reference resolution and authoring defaults;
- canonical graph representation;
- deterministic topology and trigger-rule evaluation;
- graph, node, and planner identity inputs;
- lowering from a validated graph into planner-facing structures;
- stable diagnostics for graph contract violations.

It does not own adapters, scheduler state, artifact persistence, command
routing, rendering, or repository governance.

## Internal Boundaries

| Path | Responsibility |
| --- | --- |
| `../src/graph/` | graph-domain models, composition, expansion, topology, and resources |
| `../src/pipeline/` | parse, resolve, and validate entrypoints |
| `../src/analysis/` | effects, fingerprints, semantics, and trigger rules |
| `../src/planner/` | deterministic planner lowering |
| `../src/build/` | builders and compile-oriented wrappers |
| `../src/contracts/` | kernel-owned compatibility and invariant checks |
| `../src/lib.rs` | curated exports, stable lane, prelude, and experimental lane |

Algorithms belong in their owning modules, not in the crate root. Exporting a
type does not transfer ownership away from its domain module.

## Purity Boundary

Product behavior in this crate is pure data transformation. Core source must
not read the filesystem, inspect environment variables, spawn processes,
source wall-clock time, or persist state. Serialization, hashing, allocation,
Unicode normalization, and deterministic collection operations are allowed.

A caller supplies all data needed to parse, resolve, validate, canonicalize,
fingerprint, or plan a graph.

## Identity Contract

Canonicalization must remove representation differences without erasing
semantic differences. Equal canonical graphs produce equal graph identity.
Changes to execution-relevant fields must affect the appropriate identity;
presentation-only fields must not change execution identity unless the
governing identity contract says otherwise.

Topology is deterministic for the same valid graph. Cycles, missing
dependencies, duplicate identifiers, invalid selectors, unresolved inputs, and
incompatible trigger rules are errors rather than opportunities for heuristic
repair.

## Validation Contract

Validation diagnostics carry stable identifiers, severity, location, and
actionable context where available. Validation must:

- report malformed graph structure before planner lowering;
- reject references that cannot be resolved unambiguously;
- preserve deterministic diagnostic ordering;
- avoid reading runtime state to decide graph validity;
- distinguish schema, semantic, topology, and resource failures.

The public error registry and graph data contracts govern operator-facing
codes. New diagnostics require registry, handbook, and focused test updates.

## Dependency Direction

`bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-testkit`, and `bijux-dev` may
depend on this crate. This crate must not depend on any of them.
`bijux-dag-artifacts` is a sibling data boundary; core graph meaning must not
depend on retained run layout.

## Stability

The `stable` module is the curated compatibility lane. The `prelude` supports
common graph workflows. Experimental exports require the
`experimental-public-api` feature and do not become stable merely because they
are callable.

Serialized graph shape, canonicalization, identity, diagnostics, and planner
lowering are compatibility-sensitive. A change to any of them requires
explicit fixture and downstream impact review.

## Verification

| Claim | Required evidence |
| --- | --- |
| canonical representation | `crates/bijux-dag-core/tests/canonical_contract.rs` |
| graph and node identity | `graph_identity_contract.rs` plus identity property contracts |
| deterministic topology | `graph_kernel_determinism.rs` and topology fuzz contracts |
| planner lowering | `planner_contract.rs` and planner fixture contracts |
| validation behavior | validation entrypoint, adversarial, diagnostics, and fixture contracts |
| serialized compatibility | schema round-trip, serde round-trip, and snapshot-shape contracts |

Run the focused package suite for broad kernel changes:

```bash
cargo test --locked -p bijux-dag-core
```

The normative cross-crate authorities remain under `docs/spec/`; this
crate-local page defines package ownership and does not duplicate those
behavioral specifications.
