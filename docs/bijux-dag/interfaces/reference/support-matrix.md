---
title: Support Matrix
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# Support Matrix

This matrix summarizes which `bijux-dag` surfaces are stable, experimental,
simulated, or future-facing in `v0.4.0`.

The contract source for this matrix is
[`contracts/foundation/dag_release_truth_table.v1.json`](../../../../contracts/foundation/dag_release_truth_table.v1.json)
and the handbook page
[`docs/bijux-dag/foundation/release-boundary.md`](../../foundation/release-boundary.md).

| Surface | Status | Access path | Notes |
| --- | --- | --- | --- |
| `validate`, `plan`, `run`, `replay`, `verify`, `doctor`, `version` | stable | visible CLI | primary operator surface |
| `commands` | stable | visible CLI | route inventory for stable and non-stable command discovery |
| `capabilities` | internal | `BIJUX_DAG_ENABLE_INTERNAL=1` | maintainer-only support probe outside the public operator lane |
| `prove`, `export`, `import`, `migrate inspect` | experimental | explicit-path routes | supported with narrower expectations |
| control-plane, governance, incident, lab, federation, enterprise | simulated or internal | gated routes | repository proof and modeling surfaces |
| Kubernetes, HPC, public remote scheduler service | future | not part of first-hour adoption | not a `v0.4.0` product promise |

## Primary proof

- `cargo run -p bijux-dag-cli --bin bijux-dag -- commands`
- `BIJUX_DAG_ENABLE_INTERNAL=1 cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json`
- `docs/spec/RELEASE_BINARY_VERIFICATION.md`
- `docs/bijux-dag/foundation/release-boundary.md`
