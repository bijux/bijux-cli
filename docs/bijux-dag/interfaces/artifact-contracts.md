---
title: Artifact Contracts
audience: developers
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Artifact Contracts

Retained DAG evidence must remain interpretable after the creating process,
checkout, and terminal output are gone. Trust comes from agreement between
typed records, schemas, indexes, payload bytes, and lifecycle history; the
presence of a run directory is not proof.

Use [Run Evidence Layout](run-evidence-layout.md) for exact paths. This page
defines authority, integrity, refusal, and compatibility rules for those
paths.

## Authority Order

| Surface | Authority | What it decides |
| --- | --- | --- |
| Rust data models | `bijux-dag-artifacts/src/storage/models.rs` | typed manifest, trace, index, and lifecycle meaning |
| JSON schemas | `configs/dag/schema/*.schema.json` | accepted serialized shape |
| run schema index | `run.schema.json` | schema versions and required files for one run |
| retained records | manifest, graph snapshot, traces, indexes, provenance | identity, status, dependencies, and produced artifacts |
| retained payloads | node input and output files | bytes consumed or produced |
| lifecycle records | cache proofs, lineage, retention, promotion ledgers | reuse and movement decisions |
| summaries and terminal envelopes | command output and derived reports | orientation only |

When surfaces disagree, do not select the most convenient record. Preserve the
run, report the mismatch, and refuse any claim that requires the disputed
evidence.

## Separate The Trust Questions

| Question | Evidence | A positive result does not prove |
| --- | --- | --- |
| Did evidence writing finish? | finalization marker and finalized manifest | run success |
| Did the run succeed? | manifest status and node counts | payload integrity |
| What happened to a node? | node trace and attempt records | domain correctness |
| Which bytes survived? | output index, size, digest, and payload | intended business result |
| Can cache be reused? | exact identity match plus verified cache entry | external side-effect equivalence |
| Can replay cross this boundary? | source identity, input index, payload, and digest verification | effects outside retained evidence |
| Was an artifact promoted? | promotion ledger and destination digest | approval outside the recorded policy |

These questions remain independent even when one command reports several of
them together.

## Finalization Is Not Success

Execution stages a run under `run.tmp-<run_id>` and renames it to
`run-<run_id>` after retained writing.

`manifest.finalized.json` is a copy made at finalization.
`.run-complete.json` states that evidence finalization completed and points to
that manifest. It does not state that all nodes succeeded. A failed run can be
completely finalized and valuable for diagnosis.

`.run-incomplete.json` records a reason that retained output is incomplete.
Current runtime timeout handling uses this boundary. Do not infer missing
nodes, outputs, or indexes from the manifest alone when the incomplete marker
is present.

The manifest’s `status`, `node_counts`, and failure records answer the execution
outcome question. Finalization markers answer the evidence-completeness
question.

## Identity And Integrity

An artifact identity combines more than a path:

- source run and producer node;
- declared output name and relative path;
- output kind and media type;
- content size and SHA-256 digest;
- producer and execution fingerprints;
- upstream lineage;
- schema or format version where applicable.

Paths are locators, not identities. Two files at the same relative path can
carry different producer or content identity. Two byte-identical files can
still have materially different lineage.

Verification must check the retained payload against its index and digest. It
must also check that producer, graph, planner, execution, and schema identities
are compatible with the claim being made. Hash equality alone cannot establish
that the right graph, adapter, policy, or environment produced the bytes.

## Required Refusals

Treat evidence as unverified when any required condition is missing or
inconsistent, including:

- absent or malformed manifest, trace, schema index, or output index;
- unsupported manifest, trace, index, lineage, or bundle version;
- a required payload missing from its indexed path;
- size or digest mismatch;
- output that was never declared by its producer contract;
- producer, execution, environment, or lineage identity mismatch;
- an incomplete run used where finalized evidence is required;
- a cache entry whose metadata, payload, or proof cannot be verified;
- replay input that cannot be attributed to the recorded source output;
- promotion records that disagree with the source digest or destination.

A refusal is a valid and necessary result. Do not silently downgrade strict
verification, reconstruct missing proof from logs, or label an unchecked
payload as equivalent.

## Cache And Promotion Boundaries

A cache entry is a reusable node result, not a complete run. Its manifest,
metadata, output index, payloads, and logs are governed together. Reuse requires
both identity compatibility and integrity verification. A cache hit record
with `verified = false` is not admitted reuse.

Promotion does not rewrite source identity. The run-local promotion ledger
binds source run, node, output, digest, destination, environments, timestamp,
and lineage. The compact manifest summary helps discovery; the ledger and
destination digest carry the audit claim.

Retention policy may remove material only according to an explicit plan.
Deleting a payload while retaining its index preserves a record of identity
but destroys payload-backed replay and verification capability. Reports must
state that loss instead of describing the run as fully reproducible.

## Schema Evolution

Persisted artifact changes require compatibility review when they:

- add, remove, rename, or reinterpret a field;
- change required-file rules in `RunDirSchemaIndex`;
- change a status, failure, cache, replay, or promotion vocabulary;
- alter digest inputs or canonicalization;
- change path construction or output-index semantics;
- change a schema or manifest version.

Additive optional fields still need readers that tolerate their absence.
Breaking serialized changes require a new governed version, compatibility
fixtures, migration or refusal behavior, and updates to schemas, docs, and
contract tests in the same review.

## Verification Workflow

For an operator-facing retained run:

```bash
bijux-dag explain ./artifacts/runs/run-example
bijux-dag verify --json ./artifacts/runs/run-example --strict
```

Use `explain` to orient the investigation and strict verification to establish
the retained contract. Then inspect domain output separately. A verified report
can still contain the wrong analysis, and a correct-looking report can still
lack trustworthy provenance.

## Implementation Authorities

- `crates/bijux-dag-artifacts/src/storage/models.rs`
- `crates/bijux-dag-artifacts/src/storage/hardening.rs`
- `crates/bijux-dag-artifacts/src/integrity/`
- `crates/bijux-dag-artifacts/src/lifecycle/`
- `configs/dag/schema/run_manifest.schema.json`
- `configs/dag/schema/node_trace.schema.json`
- `configs/dag/schema/inputs_index.schema.json`
- `configs/dag/schema/outputs_index.schema.json`

Continue with [Reproducibility Model](reproducibility-model.md) for identity
composition and the [Replay Contract](../../spec/REPLAY_CONTRACT.md) for the
proof boundary around reruns.
