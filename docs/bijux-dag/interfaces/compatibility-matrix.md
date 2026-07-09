---
title: Compatibility Matrix
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Compatibility Matrix

Use this page when the question is whether a retained `bijux-dag` artifact or
graph payload belongs to a supported compatibility lane.

This page is the top-level interface authority for the released compatibility
lanes. For the deeper reference table, open
[Compatibility Matrix Reference](reference/compatibility-matrix.md). For the
policy and evolution rules behind those lanes, open
[Compatibility Commitments](compatibility-commitments.md).

| Surface | Current lane | Accepted previous lane | Refused lane | Fixture root |
| --- | --- | --- | --- | --- |
| graph schema | `bijux-dag/v0.1` | `v1`, `v0.1`, `0.1` | `v9`, `bijux-dag/v9` | `evidence/compat/graph_schema/` |
| run-dir format | `run-dir/v0.1` | none | unsupported future formats | `evidence/compat/run_dir/` |
| export bundle | `export-bundle/v0.1` | none | unsupported past and future bundle versions | `evidence/compat/export_bundle/` |

The machine-readable source of truth remains
`contracts/foundation/version_compatibility_lanes.v1.json`.
