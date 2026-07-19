# `bijux-dag-artifacts` Contracts

`bijux-dag-artifacts` owns retained DAG evidence: its models, paths, integrity
proofs, persistence services, and lifecycle policy primitives. It does not
decide what to execute or whether a graph is valid.

## Owned Surface

The crate owns:

- run manifests, node traces, input and output indexes, and provenance models;
- normalized run-directory and artifact path rules;
- artifact hashing, identity, integrity proof, and schema checks;
- filesystem-backed artifact storage services;
- retention, promotion, and lineage models;
- portable path and platform normalization used by retained evidence.

Runtime code may produce values that this crate persists, but it must use this
crate's models and write services rather than redefine their serialized shape.

## Internal Boundaries

| Path | Responsibility |
| --- | --- |
| `../src/storage/` | retained models, hardening rules, and high-level services |
| `../src/io/` | filesystem reads, writes, and stores |
| `../src/integrity/` | hashes, indexes, proofs, schemas, and layout contracts |
| `../src/layout/` | relative paths and platform normalization |
| `../src/lifecycle/` | lineage, retention, and promotion policy data |
| `../src/lib.rs` | curated exports, run-directory entrypoints, and error mapping |

## Path And IO Contract

Artifact paths are normalized relative paths within an owned root. Absolute
paths, traversal outside the root, and platform-dependent ambiguity are
rejected. Writes that establish retained evidence must avoid exposing a
partially valid final object as complete.

Callers provide the owning root explicitly. This crate does not discover a
repository, select a global cache, or infer an operator's retention policy from
ambient state.

## Integrity Contract

An integrity claim binds bytes, size, identity, and available provenance.
Verification must distinguish:

- a missing artifact;
- unreadable storage;
- hash or size mismatch;
- malformed index or manifest data;
- unsupported schema;
- an unsafe path;
- incomplete lineage.

Verification cannot repair evidence while claiming only to inspect it. Repair
or promotion is an explicit operation with its own result and retained record.

## Serialization Contract

The following schemas govern serialized evidence:

- `configs/dag/schema/inputs_index.schema.json`
- `configs/dag/schema/node_trace.schema.json`
- `configs/dag/schema/outputs_index.schema.json`
- `configs/dag/schema/run_manifest.schema.json`

Readers must refuse incompatible required fields rather than silently
defaulting evidence into a valid state. Additive optional fields require
round-trip and compatibility evidence. Breaking shape changes require the
repository schema-evolution and migration authorities.

## Dependency Direction

`bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-testkit`, and `bijux-dev` may
depend on this crate. This crate must not depend on runtime, application, CLI,
or maintainer packages. It also must not import graph planner behavior from
`bijux-dag-core`; retained evidence records graph identities without owning
their computation.

## Stability

The `stable` module is the curated compatibility lane. The `prelude` groups
common retained-evidence operations. Experimental exports remain feature-gated.

Run layout, path normalization, hash interpretation, manifest shape, and
verification outcomes are compatibility-sensitive even when their Rust types
remain source-compatible.

## Verification

| Claim | Required evidence |
| --- | --- |
| artifact identity and lineage | `crates/bijux-dag-artifacts/tests/artifact_identity_and_lineage_contracts.rs` |
| run manifest identity and round trip | run-manifest identity, round-trip, and retention contracts |
| path and store safety | IO/store and storage-resilience contracts |
| corruption and hardening behavior | artifact hardening contracts |
| public compatibility lane | `crates/bijux-dag-artifacts/tests/public_api_contract.rs` |

Run the package suite for broad retained-evidence changes:

```bash
cargo test --locked -p bijux-dag-artifacts
```

Repository-wide run, storage, replay, and import/export specifications remain
under `docs/spec/`. Generated observations remain under `docs/reports/`; they
are evidence about this package, not package contracts.
