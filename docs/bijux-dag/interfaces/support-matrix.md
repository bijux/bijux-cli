---
title: Support Matrix
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Support Matrix

This matrix summarizes which `bijux-dag` surfaces are stable, experimental,
simulated, or future-facing in `v0.4.0`.

| Surface | Status | Access path | Notes |
| --- | --- | --- | --- |
| `validate`, `plan`, `run`, `replay`, `verify`, `doctor`, `version` | stable | visible CLI | primary operator surface |
| `capabilities`, `commands` | stable | visible CLI | machine-readable support summary and route inventory |
| `prove`, `export`, `import`, `migrate inspect` | experimental | explicit-path routes | supported with narrower expectations |
| control-plane, governance, incident, lab, federation, enterprise | simulated or internal | gated routes | repository proof and modeling surfaces |
| Kubernetes, HPC, public remote scheduler service | future | not part of first-hour adoption | not a `v0.4.0` product promise |

## Primary proof

- `cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json`
- `docs/spec/RELEASE_BINARY_VERIFICATION.md`
- `docs/bijux-dag/foundation/release-boundary.md`
