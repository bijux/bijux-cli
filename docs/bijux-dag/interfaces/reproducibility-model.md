---
title: Reproducibility Model
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Reproducibility Model

This reference explains how `bijux-dag` decides whether two runs describe the
same graph, the same plan, the same execution inputs, the same cache entry, or
the same retained outputs.

Use this page when the question is "what identity does this field actually
represent?" rather than "which command should I run next?"

The examples and claims on this page are grounded in the retained run evidence
documented in [Run Evidence Layout](run-evidence-layout.md), the replay
boundary rules in [`docs/spec/REPLAY_CONTRACT.md`](../../spec/REPLAY_CONTRACT.md),
the export and import rules in
[`docs/spec/IMPORT_EXPORT_CONTRACT.md`](../../spec/IMPORT_EXPORT_CONTRACT.md),
and the live runtime and app surfaces under:

- `crates/bijux-dag-runtime/src/runtime_core/planning/`
- `crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs`
- `crates/bijux-dag-runtime/src/cache/mod.rs`
- `crates/bijux-dag-app/src/cache/service.rs`
- `crates/bijux-dag-app/src/replay/service.rs`

## Identity Layers

`bijux-dag` does not use one catch-all fingerprint for every question.
Different surfaces answer different questions:

- graph fingerprint: "did the authored graph resolve to the same canonical DAG
  structure?"
- plan fingerprint: "did planning lower the graph to the same execution plan?"
- execution fingerprint: "would the runtime execute the same work under the
  same execution-relevant settings?"
- environment fingerprint: "did this node declare the same environment
  contract for cache reuse?"
- output fingerprint: "did the retained artifacts keep the same bytes and
  producer identity?"

That split is deliberate. It lets operators distinguish metadata drift,
execution drift, cache invalidation, and artifact mutation instead of collapsing
everything into a single opaque hash.

## Graph Fingerprint

The graph fingerprint is the canonical identity of the authored DAG structure.

Today it is retained in:

- `manifest.json` as `graph_fingerprint`
- `graph.snapshot.json` as `graph_fingerprint`
- replay and diff payloads that compare one run against another

It is intended to stay stable when only cosmetic ordering or non-semantic
presentation changes occur in the graph source.

It is allowed to change when the canonical graph meaning changes, including:

- nodes or edges being added or removed
- dependency ports or branch decisions changing
- node definitions changing in a way that affects canonical graph structure

The graph fingerprint answers "is this the same canonical DAG?" It does not, by
itself, prove that every execution-relevant runtime setting stayed the same.

## Plan Fingerprint

The plan fingerprint is the identity of the lowered execution plan.

The retained field name today is `planner_fingerprint`. This page calls it the
plan fingerprint because that is the operator-facing role it serves.

Today it is retained in:

- `manifest.json` as `planner_fingerprint`
- `provenance.json` as `planner_fingerprint`
- `plan.json` when that optional plan artifact is retained
- `ExecutionPlan` values built by the runtime planner

The plan fingerprint exists because graph identity and execution identity are
not the same question. Two graphs can be semantically equivalent after
canonicalization and lower to the same plan even when cosmetic metadata differs.

Current planner tests lock that behavior in place:

- `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`
- `crates/bijux-dag-runtime/tests/planner_analysis_contract.rs`

## Execution Fingerprint

The execution fingerprint is the identity of execution-relevant work.

Today it is retained in:

- `manifest.json` as `execution_fingerprint`
- `provenance.json` as `execution_fingerprint`
- `nodes/<node_id>/trace.json` as `execution_fingerprint`
- cache metadata as `node_fingerprint` for the exact executed node instance

This fingerprint is expected to change when the runtime would do materially
different work, including changes to:

- execution-relevant node parameters
- declared inputs and their lineage
- retry, timeout, trigger, cache, or branch behavior that affects execution
- policy or execution-contract settings that participate in cache reuse

It is expected to remain stable across operator-only drift such as run ids,
submission metadata, or other retained metadata that does not change execution
semantics.

Current planner and cache tests lock that split in place:

- `crates/bijux-dag-runtime/tests/planner_analysis_contract.rs`
- `crates/bijux-dag-runtime/tests/policy_cache_contract.rs`
- `crates/bijux-dag-runtime/tests/cache_evolution_contracts.rs`

## Environment Fingerprint

The environment fingerprint is the declared environment identity used for cache
reuse decisions.

The live retained field today is `declared_environment_fingerprint`.

Important boundary: there is no single run-wide `environment_fingerprint` field
in the retained manifest today. Environment identity is tracked per node where
cache reuse needs it.

Today it appears in:

- cache entry metadata `meta.json` as `declared_environment_fingerprint`
- node `trace.json` under `cache_identity.declared_environment_fingerprint`

This fingerprint tracks the node's declared runtime environment contract. It is
not a hash of the operator's full host shell, current working directory, or
ambient machine state.

That distinction matters:

- declared environment drift should invalidate cache reuse
- unrelated ambient shell drift should not silently rewrite cache identity

## Output Fingerprint

There is no single top-level retained field named `output_fingerprint` in the
standard run manifest today.

Instead, output identity is carried per artifact through:

- `sha256`
- `node_id`
- `node_fingerprint`
- retained artifact path and media metadata

Those fields live in:

- `outputs/index.json`
- `nodes/<node_id>/outputs/index.json`
- replay boundary input indexes where upstream artifacts are rematerialized

In practice, the current output identity question is answered as:

- are the retained bytes the same?
- did they come from the same execution fingerprint?
- do the retained indexes still agree with the materialized files on disk?

That is why replay, verify, repair, and cache verification all inspect artifact
hashes and producer fingerprints instead of relying on a single manifest-level
output hash.

## Cache Key

The cache key is the exact reuse identity for one node execution.

The runtime derives it from intentional inputs recorded in
`CacheKeyInput`. Today those inputs are:

- `execution_fingerprint`
- `node_definition_fingerprint`
- `declared_environment_fingerprint`
- `input_lineage_fingerprint`
- `adapter_id`
- `adapter_version`
- `output_schema_version`
- `policy_fingerprint`
- `execution_contract_fingerprint`
- `backend_class`

Those fields are hashed deterministically by the runtime cache layer and are
then persisted into cache entry metadata and cache identity reports.

The important operational split is:

- graph and plan identity explain the workflow as a whole
- cache key explains exact reuse eligibility for one node execution

That is why two runs can share a graph fingerprint while still producing a
different cache key for one node.

## Cache Verification

Cache verification does not trust the key alone.

An eligible cache entry must keep three proof surfaces aligned:

- `meta.json`
- `manifest.json`
- `outputs/index.json` plus the retained output payloads

Verification checks include:

- the stored `cache_key` matches the requested key
- persisted proof fields re-hash to the same computed cache key
- adapter identity and schema version match
- cache metadata and manifest versions are supported
- required proof fields are present
- every indexed output exists
- every indexed output hash matches the materialized file
- node fingerprints in the output index match the cache proof

When any of those checks fail, the runtime or app layer refuses reuse instead
of silently downgrading trust.

Current executable references:

- `bijux-dag cache verify`
- `bijux-dag cache explain-key`
- `bijux-dag --json why-cache-missed`
- `docs/bijux-dag/operations/cache-behavior-workflow.md`

## Replay Bundle

The portable replay bundle surface is the export bundle, not the diagnostics
bundle.

Today the supported export bundle version is `export-bundle/v0.1`.

The bundle modes matter:

- `--with-files`: the portable replay bundle surface because it carries file
  payloads as well as structural evidence
- `--manifest-only`: structural replay and inspection only; it preserves run
  shape and provenance without file payloads
- `--without-artifacts`: structural compatibility only; it keeps outputs and
  files absent on purpose

The diagnostics bundle is different:

- bundle version: `dag-diagnostics-bundle/v0.1`
- purpose: operator inspection and support capture
- not a replay import surface

Use an export bundle when the question is portability across checkouts or
origin classes. Use a diagnostics bundle when the question is "collect the
retained facts about this run for inspection."

## Replay Limitations

Replay is intentionally narrower than "run it again and hope it looks close
enough."

Current replay limitations are:

- replay depends on retained run evidence; it cannot invent missing manifests,
  traces, indexes, or artifacts
- boundary replay proves rematerialized inputs against retained hashes and node
  fingerprints, not against external business truth
- replay does not prove external side effects outside the retained artifact and
  runtime boundary
- replay sandboxing protects the source run directory from writes, but it is
  not a process-isolation or network-isolation boundary
- `--manifest-only` and `--without-artifacts` export bundles preserve structure,
  but they do not carry the full file payload needed for artifact-backed replay
- replay classification vocabulary is contract-governed; when evidence is
  missing or incompatible, replay must fail or downgrade explicitly rather than
  silently claiming equivalence

For the security boundary around replay execution, use
[Execution Security And Isolation](../operations/security-isolation-truth.md).
For the proof boundary itself, use
[`docs/spec/REPLAY_CONTRACT.md`](../../spec/REPLAY_CONTRACT.md).

## Reading Guide

- use [Run Evidence Layout](run-evidence-layout.md) when you need file paths and
  retained filesystem locations
- use [Cache Behavior Workflow](../operations/cache-behavior-workflow.md)
  when you want a full worked example of warm reuse, selective invalidation,
  corruption refusal, and cache-miss explanation
- use [Branching Bulletin Workflow](../operations/branching-bulletin-workflow.md)
  and [Compliance-Gated Bulletin Workflow](../operations/compliance-gated-bulletin-workflow.md)
  when you want replay behavior on checked-in workflow families
- use [Known Limitations](../quality/known-limitations.md) when the question
  is release-facing scope rather than identity semantics
