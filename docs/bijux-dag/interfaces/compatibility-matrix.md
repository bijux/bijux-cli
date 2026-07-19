---
title: Compatibility Matrix
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Compatibility Matrix

This matrix states which version identifiers `bijux-dag` accepts today and
which it refuses before interpreting payload contents. The machine-readable
authority is
`contracts/foundation/version_compatibility_lanes.v1.json`; this page explains
the DAG-owned subset for operators and integration authors.

## How To Read A Lane

| Classification | Meaning |
| --- | --- |
| current | canonical identifier emitted and consumed by this release |
| accepted previous | explicit alias or older identifier accepted by the current reader |
| refused | known unsupported identifier that must fail closed |

Acceptance means the reader recognizes the declared version. It does not
promise that malformed payloads, missing required evidence, or incompatible
semantics will be repaired automatically.

## DAG Compatibility Lanes

| Surface | Current identifiers | Accepted previous identifiers | Explicitly refused identifiers | Executable evidence |
| --- | --- | --- | --- | --- |
| graph schema and specification | `bijux-dag/v0.1` | `v1`, `v0.1`, `0.1` | `v9`, `bijux-dag/v9` | `evidence/compat/graph_schema/` |
| run manifest | `run-manifest/v0.1` | none | `run-manifest/v0`, `run-manifest/v2` | `evidence/compat/run_dir/` |
| run-dir format and artifact index | `run-dir-schema/v0.1` | none | `run-dir-schema/v0`, `run-dir-schema/v2` | `evidence/compat/run_dir/` |
| export bundle and proof bundle | `export-bundle/v0.1`, `proof-bundle/v0.1` | none | corresponding `v0` and `v2` bundle identifiers | `evidence/compat/export_bundle/` |

The graph spellings in the accepted column are aliases for the same retained
graph contract, not four independent schema generations. Run manifests,
artifact indexes, and replay bundles have no accepted predecessor lane in the
current contract.

## Refusal And Migration

- Reject an unknown or explicitly refused version before using its fields as
  trusted execution or replay evidence.
- Preserve the declared version in diagnostics so an operator can identify the
  incompatible producer.
- Do not silently rewrite retained runs or bundles in place.
- Use an explicit migration command or governed import path only when the
  relevant evolution rulebook defines one.
- Treat a future version as unsupported even when its JSON happens to resemble
  the current shape.

Refusal is a safety property: interpreting unknown retained evidence as current
would make replay, integrity, and compatibility claims unreliable.

## Cross-Product Boundary

The same machine contract also governs CLI command, output, and error
envelopes, configuration schema registries, and product mount descriptors.
Those are owned by the CLI handbook and are not duplicated here. A change to
the shared contract must keep both product handbooks and their executable
fixtures aligned.

## Change Review

A compatibility change must update:

- `contracts/foundation/version_compatibility_lanes.v1.json`
- the relevant schema or evolution rulebook under `docs/spec/`
- compatibility fixtures under `evidence/compat/`
- the reader or migration implementation
- this matrix and its governing tests

Adding an accepted predecessor is a deliberate compatibility expansion.
Removing one or reinterpreting an existing identifier is incompatible and
requires explicit release treatment.

## Related Contracts

- [Compatibility Commitments](compatibility-commitments.md)
- [Graph Schema Reference](graph-schema.md)
- [Run Evidence Layout](run-evidence-layout.md)
- [Reproducibility Model](reproducibility-model.md)
- [Migration Policy](../../spec/MIGRATION_POLICY.md)
