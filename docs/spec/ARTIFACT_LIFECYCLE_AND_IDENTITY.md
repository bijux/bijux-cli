# ARTIFACT LIFECYCLE AND IDENTITY

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/ARTIFACT_BUNDLE_FORMAT_V1.md
# Artifact Bundle Format v1

## Identifier

`artifact-bundle/v1`

## Required fields

- `bundle_version`: `export-bundle/v0.1`
- `format`: `artifact-bundle/v1`
- `outputs`

## Optional fields

- `files`
- `provenance`

## Invariants

- If `files` is present, each file entry must map to a stable output path key.
- If exported with `without-artifacts`, `outputs` must be an empty map and `files` must be null.

## SOURCE: docs/spec/ARTIFACT_BUNDLE_MANIFEST_EXAMPLES.md
# Artifact Bundle Manifest Examples

## Minimal

```json
{
  "pack_manifest_version": "artifact-pack/v0.1",
  "artifacts": ["n1:result.json"]
}
```

## Multi-artifact with replay ancestry context

```json
{
  "pack_manifest_version": "artifact-pack/v0.1",
  "artifacts": [
    "extract:raw.csv",
    "transform:clean.csv",
    "train:model.bin"
  ]
}
```

`artifacts` order is canonical and stable for deterministic export diffs.


## SOURCE: docs/spec/ARTIFACT_DIFF_SEMANTICS.md
# Artifact Diff Semantics

Artifact diff compares identity and lineage-aware payload evidence:

- `artifact_id`
- producer node fingerprint
- payload `sha256`
- upstream/downstream lineage references

Equivalent artifacts may have distinct provenance context across runs.

## SOURCE: docs/spec/ARTIFACT_DURABILITY_GUARANTEES_CONTRACT.md
# Artifact Durability Guarantees Contract

## Purpose

Define required durability and safety guarantees for artifact writes, reads, recovery, corruption handling, rebuild behavior, retention correctness, and verification safety.

## Required durability coverage

- artifact write atomicity and read consistency
- partial-write and corruption recovery behavior
- concurrent-write and GC race safety
- checksum verification and anomaly detection
- artifact store rebuild, compaction, and fragmentation behavior
- retention durability and lifecycle recovery
- durability benchmarks, telemetry, and stress verification

## Required governance artifacts

- artifact durability regression corpus
- artifact durability verification suite
- artifact durability benchmark report
- artifact durability telemetry report
- artifact durability anomaly report
- artifact durability coverage report

## SOURCE: docs/spec/ARTIFACT_IDENTITY_CONTRACT.md
# Artifact Identity Contract

This contract defines the authoritative artifact identity model for `bijux-dag`.

## Identity type

- `ArtifactId` is a first-class type in `crates/bijux-dag-artifacts/src/integrity/index.rs`.
- Canonical string form: `<node_id>:<file_name>`.
- Identity explanation output is emitted by `dag artifact-inspect` via
  `crates/bijux-dag-app/src/lib.rs::inspect_artifact`.

## Fingerprint composition

Artifact fingerprint is composed from:

- content digest (`sha256`)
- producing `run_id`
- producing `node_id`
- producing `node_fingerprint`
- logical artifact path within the run directory

Implementation anchors:
- `crates/bijux-dag-app/src/lib.rs::inspect_artifact`
- `crates/bijux-dag-artifacts/src/integrity/hash.rs`
- `crates/bijux-dag-artifacts/src/storage/models.rs` (`RunOutputFile`)

## Provenance links

Artifact inspection surfaces must include provenance links to:

- graph fingerprint
- run id
- node id and node fingerprint
- execution attempt number

## Logs policy

- `stdout.log` and `stderr.log` are retained as run evidence and diagnostics artifacts.
- Logs are never treated as canonical semantic outputs for replay equivalence.

## Store capabilities

Current capability status:

- filesystem store: implemented
- object store: modeled-only (not implemented in runtime)

## SOURCE: docs/spec/ARTIFACT_IDENTITY_PROVENANCE_MAPPING.md
# Artifact Identity to Provenance Mapping

This mapping defines which artifact fields represent content identity and which represent provenance context.

| Field | Category | Meaning |
| --- | --- | --- |
| `artifact_id` | identity | Logical artifact selector (`node_id:file_name`) used by operator surfaces. |
| `artifact_sha256` | identity | Content-addressed digest over payload bytes. |
| `node_fingerprint` | identity | Node execution identity that produced the artifact payload. |
| `path` | identity | Normalized relative payload location within the run directory. |
| `provenance.run_id` | provenance | Finalized run directory identity that produced/imported the artifact. |
| `provenance.graph_fingerprint` | provenance | Graph-level identity context for provenance traversal and replay checks. |
| `provenance.attempt` | provenance | Attempt lineage marker for retries or replay-derived materialization. |
| `lineage.upstream_artifact_ids` | provenance | Immediate upstream artifact dependencies for explain/trace flows. |
| `lineage.downstream_artifact_ids` | provenance | Immediate downstream artifacts that depend on this artifact. |

Boundary rule:
- Identity fields are used for stable equality and cache/replay compatibility checks.
- Provenance fields are used for ancestry, explainability, and operator audit flows.
- Provenance changes must not be interpreted as payload identity changes unless identity fields also change.

## SOURCE: docs/spec/ARTIFACT_INTEGRITY_SUITE.md
# Artifact integrity suite

## Scope

Defines the artifact integrity verification suite for run-directory storage, manifest validation, corruption detection, and replay/import-export artifact checks.

## Primary verification surfaces

- `crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs`
- `crates/bijux-dag-artifacts/tests/conformance.rs`
- `evidence/fault/corrupt_runs/*`

## Control-plane enforcement

- suite id: `artifact-hardening`
- command: `bijux-dev-dag run-dir-audit --run-dir <path> [--strict]`

## Coverage intent

- manifest validation
- run import/export payload validation
- corruption fixture rejection
- replay artifact payload verification
- retention-aware cleanup planning

## SOURCE: docs/spec/ARTIFACT_LIFECYCLE.md
# Artifact lifecycle

## States

- `staging`: run data is being written.
- `incomplete`: run interrupted before manifest finalization.
- `finalized`: manifest and finalization markers written.
- `retained`: data kept by retention policy.
- `pruned`: removable non-retained data cleaned.

## Guarantees

- Finalized runs are immutable.
- Incomplete runs are explicitly marked.
- Cleanup planning must preserve retained prefixes.

## SOURCE: docs/spec/ARTIFACT_LINEAGE_COMPLETENESS_CONTRACT.md
# Artifact Lineage Completeness Contract

## Purpose

This contract formalizes artifact lineage guarantees across production, replay,
import/export, traversal, persistence, and garbage-collection safety.

## Lineage Guarantees

- artifact provenance fields are complete and stable
- parent-child lineage relations are explicit
- upstream/downstream traversal is deterministic
- lineage persistence survives repeated inspection
- lineage reconstruction remains correct under partial runs
- replay and imported runs preserve lineage semantics
- lineage-safe GC preserves referenced artifacts

## Verification Expectations

- lineage reconstruction tests
- partial-run lineage completeness tests
- replay lineage correctness tests
- import lineage correctness tests
- GC lineage safety tests
- lineage serialization stability tests
- traversal benchmark and consistency checks
- corruption detection, fuzzing, anomaly coverage
- explainability and visualization data generation coverage


## SOURCE: docs/spec/ARTIFACT_OWNERSHIP_TABLE.md
# Artifact ownership table

| Surface | Owner crate | Owner role |
|---|---|---|
| run directory layout | `bijux-dag-artifacts` | artifact maintainers |
| manifest structure | `bijux-dag-artifacts` | artifact maintainers |
| run finalization markers | `bijux-dag-artifacts` | runtime + artifact maintainers |
| run verify and audit commands | `bijux-dev-dag` | control-plane maintainers |
| runtime manifest emission | `bijux-dag-runtime` | runtime maintainers |

## SOURCE: docs/spec/ARTIFACT_PLATFORM.md
# Artifact platform contracts

This document defines artifact-platform contracts for multi-store storage, exchange, lineage operations, and verification governance.

## Store routing and replication

- `ArtifactStoreClass`: `HotCache`, `DurableLocal`, `RemoteObject`.
- `ArtifactStoreRoute` binds logical `ArtifactId` to an explicit store class and storage key.
- `ArtifactReplicationRule` and `ArtifactReplicationRecord` define deterministic replication and promotion evidence.

## Packing, compression, and chunking

- `ArtifactPackingProfile` defines policy surfaces for replay, archive, compliance, and handoff.
- `ArtifactCompressionPolicy` encodes deterministic compression requirements.
- `ArtifactChunkPolicy` and `ArtifactChunkDescriptor` define chunking without losing content identity observability.

## Provenance, verification, and trust hooks

- `ArtifactSigningHook` reserves manifest-signing integration points.
- `ArtifactProvenanceRecord` extends provenance with producer binary identity, adapter version, and environment class.
- `ArtifactVerificationReport` provides release and audit-friendly verification output.

## Retention and safe collection planning

- `ArtifactRetentionClass` supports operational/legal classes: transient, retained, release, audit.
- `ArtifactGarbageCollectionPlan` separates preserved artifacts from collectable artifacts while tying decisions to lineage snapshot identity.

## Import/export and sensitive data controls

- `ArtifactImportCompatibility` records compatibility checks across source/target spec versions and environments.
- `ArtifactExportProfile` covers handoff, backup, replication, and compliance evidence bundles.
- `ArtifactRedactionPolicy` defines log/metadata redaction controls.
- `ImmutableArtifactAnnotation` provides operator-added context without mutating content identity.

## Lineage query and replay assist

- `compact_lineage` builds a producer-oriented compact lineage index.
- `lineage_dependencies` and `lineage_dependents` support “what produced this” and “what depends on this”.
- `build_replay_assist` emits minimal upstream context for selected artifact replay.

## Store conformance

`run_store_conformance` defines a baseline backend contract:

- write succeeds
- read succeeds
- read content equals written content

Backends are expected to pass this baseline before being promoted for runtime support.

## SOURCE: docs/spec/ARTIFACT_RETENTION_POLICY.md
# Artifact retention and cleanup policy

## Scope

This policy governs run artifacts, cache entries, promoted artifacts, and exported bundles.

## Retention classes

- `local cache`: short-lived, evictable, rebuildable.
- `run artifacts`: medium-lived evidence for replay and diagnostics.
- `promoted artifacts`: long-lived evidence and release assets.
- `exported bundles`: transport artifacts with bounded retention.

## Baseline defaults

Default retention values are defined by `RetentionPolicy` in
`crates/bijux-dag-artifacts/src/retention.rs`:

- local cache: 7 days
- run artifacts: 30 days
- promoted artifacts: 365 days
- exported bundles: 180 days

## Cleanup rules

- Cleanup must never delete active staging directories.
- Cleanup must never delete promoted artifacts before promoted retention expires.
- Cleanup must preserve run manifests and trace files for runs still within run-artifacts retention.
- Cleanup must be deterministic for equal input state and cutoff timestamps.

## Deferred implementation boundary

Garbage collection orchestration is documented and policy-backed, but full automated GC
execution can be deferred. Any future GC implementation must enforce this policy and emit
audit evidence for each deletion decision.

## SOURCE: docs/spec/ARTIFACT_STORAGE_LIFECYCLE_CONTRACT.md
# Artifact Storage Lifecycle Contract

## Purpose

Define durable behavior for artifact creation, storage, retrieval, retention, and cleanup.

## Lifecycle expectations

- creation -> storage -> retrieval is lossless for accepted writes
- replay and imported runs preserve artifact lineage context
- partial reruns preserve valid historical artifact references
- retention and garbage-collection decisions are explainable and deterministic

## Integrity and recovery requirements

- index consistency between manifest and output index files
- checksum verification for stored artifacts
- corruption detection for payload and metadata failures
- recovery guidance after partial writes and interrupted operations
- repair guidance for index mismatches and stale paths

## Safety constraints

- retention enforcement across ancestry chains
- garbage collection safety under concurrent writes
- garbage collection safety during replay
- fragmentation and storage health detection remain observable

## Governance artifacts

- regression corpus for lifecycle and corruption cases
- lifecycle stress suite
- lifecycle benchmark and telemetry reports

## SOURCE: docs/spec/ARTIFACT_SYSTEM.md
# Artifact system contracts

## Module boundaries

`bijux-dag-artifacts` is organized into explicit modules:

- `store`: filesystem/object storage backend contracts
- `models`: run artifact schemas and wire models
- `paths`: canonical artifact-relative path helpers
- `index`: artifact ids, aliases, output classes, pack manifest, dedup metrics
- `hash`: content hashing helpers
- `schema`: output schema descriptor and validation hook
- `lineage`: lineage edge model and snapshot writer
- `retention`: retention policy model
- `promotion`: promotion record and environment model
- `proof`: integrity proof and corruption policy models

## Reproducibility verification

Use `bijux-dev-dag` to verify local artifact reproducibility:

- `cargo run -p bijux-dev-dag -- artifact-verify`

This command checks manifest output hashes against on-disk output files.

For full integrity checks of a run directory, use:

- `dag verify <run-dir> --deep`

Deep verification validates canonical output index ordering, normalized relative artifact paths,
and trace/manifest schema parseability from stored JSON evidence.

## Lifecycle policy

Artifact retention and cleanup policy is defined in:

- `crates/bijux-dag-artifacts/src/retention.rs` (typed policy model)
- `docs/spec/ARTIFACT_RETENTION_POLICY.md` (operator-facing retention and cleanup rules)

## SOURCE: docs/spec/CANONICAL_GRAPH_IDENTITY_SPEC.md
# Canonical Graph Identity Specification

## Purpose

Define the canonical, deterministic identity for a graph as the hash of canonical graph JSON.

## Identity definition

- Canonical algorithm version: `bijux-dag-canonical/v1`
- Hash algorithm: `sha256`
- Graph identity value: `sha256(canonical_graph_json_bytes)`

## Canonicalization rules

1. Parse graph with strict schema (`serde` unknown fields rejected).
2. Normalize accepted spec aliases to `bijux-dag/v0.1`.
3. Normalize path separators to `/` for output paths.
4. Normalize Unicode identity text fields using NFC.
5. Sort nodes by `id`.
6. Sort node inputs.
7. Sort node outputs by output `name`.
8. Sort effects by stable effect order.
9. Sort env allowlist and tags.
10. Sort edges by `(from.node_id, from.port, to.node_id, to.port)`.
11. Sort JSON object keys in graph inputs and resolved params recursively.
12. Treat explicit zero resources (`cpu=0`, `mem_mb=0`) as absent resources.

## Semantic vs non-semantic changes

Non-semantic (must not change identity):
- JSON key ordering
- YAML key ordering after YAML->JSON normalization
- Whitespace-only formatting changes
- Comment-only source changes when comments are stripped before strict parse
- Edge list permutation

Semantic (must change identity):
- Node command/params changes
- Dependency edge topology changes
- Resource specification changes

## Backend independence

Graph identity is derived from canonical graph content only; runtime backend execution path is outside identity derivation.

## Contract surfaces

- Core API: `Graph::graph_id`, `Graph::graph_fingerprint_explain`
- CLI: `dag hash graph`, `dag fingerprint --explain`

## SOURCE: docs/spec/GRAPH_BUNDLE_FORMAT_V1.md
# Graph Bundle Format v1

## Identifier

`graph-bundle/v1`

## Required fields

- `bundle_version`: `export-bundle/v0.1`
- `format`: `graph-bundle/v1`
- `graph_snapshot`: canonical DAG snapshot
- `manifest`: export manifest metadata

## Optional fields

- `provenance`
- `notes`

## Invariants

- `graph_snapshot.spec` must be parseable as supported DAG schema.
- Graph identity derived from `graph_snapshot` must remain canonical across import/export.

## SOURCE: docs/spec/RELEASE_ARTIFACTS.md
# Release Artifacts

## Scope
Defines the expected artifact set for each release.

## Required artifacts
- binaries
- checksums
- schemas snapshot
- docs snapshot
- benchmark summary
- resource profile summary
- compatibility matrix
- known limitations note
- release readiness report
- release evidence bundle

## Related tests
- `crates/bijux-dev-dag/src/commands/mod.rs` (`run_release_readiness_report`, `run_release_evidence_bundle`)

## Versioning and change policy
Artifact set removals are breaking for release consumers and require policy update.

## SOURCE: docs/spec/RUN_ARTIFACT_SPEC_V0.1.md
# Run Artifact Spec v0.1

Schema source of truth:
- `configs/schema/run_manifest.schema.json`
- `configs/schema/node_trace.schema.json`
- `configs/schema/outputs_index.schema.json`

## Run Directory Layout
```
run-<id>/
  manifest.json
  provenance.json
  graph.snapshot.json
  outputs/
    index.json
  nodes/
    <node_id>/
      trace.json
      resolved_params.json
      stdout.log
      stderr.log
      inputs/
        index.json
        <files>
      outputs/
        index.json
        <files>
```

## manifest.json
```
{
  "run_id": "string",
  "created_unix_ms": number,
  "graph_snapshot": "graph.snapshot.json",
  "status": "success|failed|cancelled",
  "spec": "bijux-dag/v0.1"
}
```

## graph.snapshot.json
```
{
  "graph": <canonical graph>,
  "graph_fingerprint": "sha256"
}
```

## trace.json
```
{
  "node_id": "string",
  "status": "success|failed|skipped|cached",
  "started_unix_ms": number,
  "finished_unix_ms": number,
  "fingerprint": "sha256"
}
```

## provenance.json
```
{
  "os": "string",
  "arch": "string",
  "rustc": "string",
  "tool_version": "string",
  "adapters": [...],
  "policy": {...},
  "time_source": "system_clock"
}
```

## run outputs/index.json
```
{
  "files": [
    {"node_id": "id", "node_fingerprint": "...", "sha256": "...", "path": "nodes/<id>/outputs/file"}
  ]
}
```

## resolved_params.json
Resolved parameters for the node with deterministic key ordering.

## inputs/index.json
```
{
  "files": [
    {"path": "upstream/input_name", "sha256": "...", "from_node": "id", "from_node_fingerprint": "...", "from_output": "port"}
  ]
}
```

## outputs/index.json
```
{
  "files": [
    {"path": "file", "sha256": "...", "node_id": "id", "node_fingerprint": "..."}
  ]
}
```

## SOURCE: docs/spec/RUN_VS_ARTIFACT_LINEAGE.md
# Run Lineage vs Artifact Lineage

Run lineage and artifact lineage are related but not interchangeable.

## Run Lineage

Run lineage explains **which run came from which run**.

Primary fields:
- `run_id`: immutable identity of the finalized run directory.
- `run_metadata.parent_run_id`: immediate replay/import parent in run ancestry.
- `run_metadata.source_run_id`: authoritative source run used for replay or import.

Use run lineage for:
- replay ancestry and run-tree navigation,
- run history and operator timeline context,
- provenance of run-level verification reports.

## Artifact Lineage

Artifact lineage explains **which node/run produced which artifact**.

Primary fields:
- artifact identity (`sha256`, artifact id, logical output path),
- producer (`run_id`, `node_id`),
- upstream/downstream artifact edges.

Use artifact lineage for:
- trace-artifact and artifact-inspect workflows,
- retention and GC safety decisions,
- semantic diff and replay mismatch root-cause analysis.

## Boundary Rule

Run lineage must never be used as a substitute for artifact lineage, and artifact lineage must never be treated as run ancestry. The two surfaces are queried together but validated independently.
