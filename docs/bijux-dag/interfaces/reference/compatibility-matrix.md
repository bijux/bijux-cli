---
title: Compatibility Matrix
audience: mixed
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Compatibility Matrix

This matrix records the compatibility lanes for the core versioned surfaces in
`bijux-dag`.

| Surface | Current lane | Accepted previous lane | Refused lane | Fixture root |
| --- | --- | --- | --- | --- |
| graph schema | `bijux-dag/v0.1` | `v1`, `v0.1`, `0.1` | `v9`, `bijux-dag/v9` | `evidence/compat/graph_schema/` |
| run-dir format | `run-dir/v0.1` | none | unsupported future formats | `evidence/compat/run_dir/` |
| export bundle | `export-bundle/v0.1` | none | unsupported past and future bundle versions | `evidence/compat/export_bundle/` |

The machine-readable source of truth remains
`contracts/foundation/version_compatibility_lanes.v1.json`.
