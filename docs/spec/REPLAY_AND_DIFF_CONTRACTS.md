# REPLAY AND DIFF CONTRACTS

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/DIFF_CLASSIFICATION_CONTRACT.md
# Diff Classification Contract

`bijux-dag` distinguishes two classes:

- `semantic`: execution meaning changes (identity/fingerprint/outcomes/payload hashes)
- `cosmetic`: representation-only changes with no execution meaning change

Operator surfaces:

- `dag diff`
- `dag why-rerun`
- `dag trace-artifact`

must present semantic-cause summaries in machine-readable form.

## SOURCE: docs/spec/GRAPH_DIFF_SEMANTICS.md
# Graph Diff Semantics

Graph diff classifies changes into:

- semantic: changes that alter graph identity or execution behavior
- cosmetic: formatting/order/comment changes that do not alter graph identity

Canonical graph bytes are the source of truth for semantic comparison.

## SOURCE: docs/spec/GRAPH_DIFF_SPEC_V0.1.md
# Graph Diff Spec v0.1

## Scope

Graph diff compares two DAG definitions at canonical semantic level.

## Inputs

- canonical graph bytes for graph A
- canonical graph bytes for graph B
- graph fingerprints for graph A and graph B

## Classification

- `semantic_change`: canonical graph bytes differ
- `cosmetic_only`: raw source text differs but canonical graph bytes are equal
- `equivalent`: canonical graph bytes are equal and fingerprints are equal

## Required Output Fields

- `equivalent` (boolean)
- `graph_fingerprint` (object or null)
- `reason_report.summary` (string)
- `cause_groups` (object)

## Determinism Requirements

- identical inputs MUST produce byte-identical JSON output
- field ordering MUST remain stable
- cause group naming MUST remain stable across patch releases

## Non-Goals

- runtime timing comparison
- resource usage attribution
- backend conformance evaluation

## SOURCE: docs/spec/REPLAY_CONTRACT.md
# Replay contract

## Scope
Defines replay semantics, semantic diff interpretation, provenance boundaries, and explicit non-goals.

## Replay definition
Replay in this repository means:

- execute from captured graph + runtime artifacts
- compare semantic outputs and node outcomes against prior run evidence
- emit explicit reason report for equivalence or mismatch

Replay is not a byte-for-byte filesystem restore mechanism.

## Authoritative inputs

Replay may consult:

- `manifest.json`
- graph snapshot and graph fingerprint
- node traces and statuses
- outputs index and output hashes
- provenance markers and replay source metadata

Replay must not consult ambient host state as authoritative evidence.

## Semantic diff mode

`dag diff` and `dag runs diff` support semantic comparison mode and emit:

- replay equivalence boolean
- mismatch reasons
- grouped mismatch causes
- replay reason summary

## Replay explain mode

`--explain` output groups mismatch causes by class:

- `manifest_drift`
- `graph_semantics`
- `node_outcomes`
- `artifact_payload`

## Fixture families

Replay fixture family includes:

- `evidence/cache/replay/match_case.json`
- `evidence/cache/replay/mismatch_case.json`
- `evidence/cache/replay/corruption_case.json`
- `evidence/cache/replay/unsupported_version_case.json`

## What replay cannot prove

Replay cannot prove:

- equivalence to uncaptured external side effects
- equivalence when authoritative artifacts are missing
- compatibility across unsupported historical or future formats
- equivalence of non-semantic metadata fields intentionally ignored by contract

## Related tests

- `tests/e2e/replay/*`
- `crates/bijux-dag-app/tests/e2e_integration_scenarios.rs`
- `crates/bijux-dag-app/tests/replay_contract.rs`
- `crates/bijux-dag-app/src/diff.rs` unit tests

## Related schemas

- `configs/schema/operator/replay_diff.schema.json`
- `configs/schema/run_manifest.schema.json`
- `configs/schema/node_trace.schema.json`

## Versioning and change policy
Replay semantics changes require explicit compatibility decision and updated replay fixture and schema coverage.

## SOURCE: docs/spec/REPLAY_EQUIVALENCE_COMPLETENESS_CONTRACT.md
# Replay Equivalence Completeness Contract

## Purpose

This contract defines replay equivalence guarantees, mismatch classification,
fidelity levels, drift semantics, and verification surfaces.

## Replay Guarantees

- replay correctness and deterministic planning guarantees
- mismatch classification model coverage
- fidelity level semantics and reporting guarantees
- environment and artifact drift semantics
- replay proof generation and verification guarantees

## Verification Expectations

- replay equivalence detection tests
- mismatch classification tests
- fidelity reporting tests
- deterministic planning tests
- replay proof verification tests
- regression corpus, fuzzing, anomaly detection, benchmarks
- explainability, telemetry, diagnostics coverage


## SOURCE: docs/spec/REPLAY_EVIDENCE_CONTRACT.md
# Replay Evidence Contract

Replay evidence in proof output captures availability and completeness of replay-related run artifacts.

Required replay fields:
- `replay_evidence.available`
- `replay_evidence.level`

## SOURCE: docs/spec/REPLAY_FIDELITY_LEVELS.md
# Replay Fidelity Levels

Replay fidelity is reported with explicit levels:

- `strict_equivalent`: replay result is semantically equivalent to source run across manifest, graph fingerprint, node outcomes, and output hashes.
- `diverged`: one or more trust dimensions differ and replay proof reports the mismatch reasons.

Implementation anchors:
- `crates/bijux-dag-app/src/replay/diff.rs` (`ReplayEquivalence`, mismatch grouping and reason report)
- `crates/bijux-dag-app/src/lib.rs` (`dag replay --prove` JSON + human-readable proof surface)
- `configs/schema/operator/replay_proof.schema.json` (wire-format contract)

`replay` means re-executing from recorded run evidence and checking fidelity.
`rerun` means a new execution without required equivalence proof.

## SOURCE: docs/spec/SEMANTIC_DIFF_COMPLETENESS_CONTRACT.md
# Semantic Diff Completeness Contract

## Purpose

This contract defines semantic diff guarantees for graph, run, artifact,
environment, and backend capability differences.

## Diff Guarantees

- graph diff semantics are explicit and deterministic
- run diff semantics are explicit and deterministic
- artifact diff semantics are explicit and deterministic
- environment diff semantics are explicit and deterministic
- backend capability diff semantics are explicit and deterministic
- diff classification outputs are consistent and explainable

## Verification Expectations

- semantic correctness tests
- classification consistency tests
- determinism tests across runs and platforms
- regression corpus and fuzzing coverage
- anomaly detection, benchmark, telemetry, and diagnostics coverage
- visualization data generation and documentation coverage

