---
title: Reproducibility Model
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Reproducibility Model

Bijux DAG uses separate identities for authored structure, planned work,
executed work, declared environment, cache eligibility, and retained output.
No single fingerprint proves that two runs are equivalent.

Behavioral authority:

- retained layout: [Run Evidence Layout](run-evidence-layout.md);
- replay semantics: `docs/spec/REPLAY_CONTRACT.md`;
- bundle semantics: `docs/spec/IMPORT_EXPORT_CONTRACT.md`;
- planner, execution, and cache code:
  `crates/bijux-dag-runtime/src/runtime_core/` and
  `crates/bijux-dag-runtime/src/cache/`.

## Identity Matrix

| Identity | Question | Retained evidence | Does not prove |
| --- | --- | --- | --- |
| graph | Is canonical authored structure equal? | `graph_fingerprint` | same plan, environment, or output |
| plan | Did lowering produce the same plan? | `planner_fingerprint` | same runtime inputs or artifacts |
| execution | Is execution-relevant work equal? | `execution_fingerprint` and node fingerprints | same external side effects |
| environment | Is the declared node environment equal? | `declared_environment_fingerprint` | same ambient host |
| cache key | Is one node result eligible for exact reuse? | cache `meta.json` and manifest | retained payload integrity by itself |
| output | Are bytes and producer identity equal? | artifact digest and producer fingerprint | business-level equivalence |

## Graph Fingerprint

The graph fingerprint identifies canonical DAG structure. It appears in
`manifest.json`, `graph.snapshot.json`, and replay/diff payloads.

It changes for canonical node, edge, port, branch, or node-definition changes.
It can remain stable across ordering or presentation differences that
canonicalization removes.

Equal graph fingerprints answer only “same canonical graph.” Runtime policy,
effective inputs, environment, and retained bytes require their own evidence.

## Plan Fingerprint

The retained field is `planner_fingerprint`. It identifies the lowered
execution plan and appears in:

- `manifest.json`;
- `provenance.json`;
- optional `plan.json`;
- runtime `ExecutionPlan` values.

Plan identity separates authored structure from executable lowering. Cosmetic
graph metadata may differ while the lowered plan remains equal. Conversely,
planner policy can change execution meaning even when source structure looks
similar.

## Execution Fingerprint

`execution_fingerprint` identifies execution-relevant work. It is retained in
the run manifest, provenance, node traces, and cache identity.

It changes when execution meaning changes, including:

- effective parameters or input lineage;
- retry, timeout, trigger, cache, or branch behavior;
- policy or execution-contract inputs;
- adapter or backend meaning that participates in execution identity.

Run IDs, submission labels, and non-execution metadata should not alter it.
Equality does not prove that an external service produced the same side effect.

## Environment Fingerprint

Environment identity is node-scoped, not one run-wide host hash. The field
`declared_environment_fingerprint` appears in cache `meta.json` and node
`trace.json` cache identity.

It covers the declared environment contract used for reuse. It does not hash
the operator’s entire shell, working directory, machine, or undeclared ambient
state.

Declared environment drift should invalidate reuse. Undeclared ambient
dependencies are a reproducibility defect, not inputs the cache can safely
infer.

## Output Fingerprint

There is no standard top-level `output_fingerprint`. Artifact identity is
carried per output through:

- `sha256`;
- producer `node_id`;
- producer `node_fingerprint`;
- retained path, kind, and media metadata.

Those values live in `outputs/index.json`,
`nodes/<node_id>/outputs/index.json`, and replay-boundary input indexes.

Output equivalence requires matching bytes and compatible producer evidence.
Matching paths or names are insufficient.

## Cache Key

The cache key is exact reuse identity for one node. `CacheKeyInput` includes:

- execution and node-definition fingerprints;
- `declared_environment_fingerprint`;
- input-lineage fingerprint;
- adapter ID and version;
- output schema version;
- policy and execution-contract fingerprints;
- backend class.

Graph equality does not imply cache equality. One node can miss while the rest
of an unchanged graph remains reusable.

## Cache Verification

A key identifies a candidate entry; verification decides whether it is safe to
reuse. Three surfaces must agree:

- `meta.json`;
- cache `manifest.json`;
- `outputs/index.json` and materialized payloads.

Verification checks:

- requested and stored keys agree;
- persisted proof fields re-hash to that key;
- adapter, schema, manifest, and metadata versions are supported;
- required proof fields exist;
- indexed outputs exist and their digests match;
- output producer fingerprints agree with cache proof.

Failure refuses reuse rather than silently reducing assurance. Use
`bijux-dag cache verify` for integrity and the tested explicit-path
`why-cache-missed` route for one node’s decision.

## Replay Bundle

The portable replay surface is `export-bundle/v0.1`, governed by
`docs/spec/IMPORT_EXPORT_CONTRACT.md`.

| Mode | Carries | Honest use |
| --- | --- | --- |
| `with-files` (`--with-files`) | structure, provenance, and file payloads | artifact-backed portable replay |
| `manifest-only` (`--manifest-only`) | structural evidence without files | inspection and structural compatibility |
| `without-artifacts` (`--without-artifacts`) | intentionally omitted artifacts | structure-only transfer |

`dag-diagnostics-bundle/v0.1` is for inspection and support capture. It is not
an importable replay bundle.

## Replay Limitations

Replay can compare only retained evidence. It cannot invent missing manifests,
traces, indexes, inputs, or artifacts.

Further boundaries:

- rematerialized inputs are checked against retained digests and producer
  identity, not external business truth;
- external side effects outside retained evidence are not reproduced or proven;
- replay `--sandbox` protects the source run directory from writes but is not
  process, network, or host isolation;
- `manifest-only` and `without-artifacts` lack payloads required for
  artifact-backed proof;
- missing, corrupt, or incompatible evidence must cause explicit refusal or
  reduced classification, never silent equivalence.

Use [Execution Security And Isolation](../operations/security-isolation-truth.md)
for host boundaries and `docs/spec/REPLAY_CONTRACT.md` for normative replay
classification.

## Review Questions

Before claiming reproducibility, record:

1. Which identity layer is being compared?
2. Which retained fields and files support it?
3. Were payload digests recomputed?
4. Which environment and backend inputs participate?
5. Are external effects outside the retained boundary?
6. Is the bundle artifact-bearing or structural only?

## Continue By Question

- file locations: [Run Evidence Layout](run-evidence-layout.md);
- cache behavior: [Cache Behavior Workflow](../operations/cache-behavior-workflow.md);
- active scope limits: [Known Limitations](../quality/known-limitations.md);
- replay contract: `docs/spec/REPLAY_CONTRACT.md`;
- import/export contract: `docs/spec/IMPORT_EXPORT_CONTRACT.md`.
