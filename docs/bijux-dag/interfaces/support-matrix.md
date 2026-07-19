---
title: Support Matrix
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Support Matrix

Use this page when the question is whether a `bijux-dag` surface is part of
the shipped operator promise, an internal maintainer probe, or an unreleased
boundary.

For the detailed reference version, open
[Support Matrix Reference](support-matrix.md). For the governing
release policy, open
[Release Boundary](../foundation/release-boundary.md).
The [Known Limitations](../quality/known-limitations.md) page records current
constraints. The [Bijux Dag Roadmap](../roadmap.md) is
future direction, not a supported-surface claim.

| Surface | Status | Access path | Notes |
| --- | --- | --- | --- |
| `validate`, `plan`, `run`, `replay`, `verify`, `doctor`, `version` | stable | visible CLI | primary operator surface |
| `commands` | stable | visible CLI | route inventory for stable and non-stable command discovery |
| `capabilities` | internal | `BIJUX_DAG_ENABLE_INTERNAL=1` | maintainer-only support probe outside the public operator lane |
| `prove`, `export`, `import`, `migrate inspect` | experimental | explicit-path routes | supported with narrower expectations |
| control-plane, governance, incident, lab, federation, enterprise | simulated or internal | `commands --lane simulated` plus opt-in env for execution | repository proof and modeling surfaces |

## Primary proof

- `cargo run -p bijux-dag-cli --bin bijux-dag -- commands`
- `BIJUX_DAG_ENABLE_INTERNAL=1 cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json`
- `docs/spec/RELEASE_BINARY_VERIFICATION.md`
- `docs/bijux-dag/foundation/release-boundary.md`
