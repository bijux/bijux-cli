---
title: Support Matrix
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Support Matrix

Use this page when the question is whether a `bijux-dag` surface is part of
the shipped operator promise, an internal maintainer probe, or an unreleased
boundary.

The governing policy is the
[Release Boundary](../foundation/release-boundary.md), backed by
`contracts/foundation/dag_release_truth_table.v1.json`.
The [Known Limitations](../quality/known-limitations.md) page records current
constraints. The [Bijux Dag Roadmap](../roadmap.md) is
future direction, not a supported-surface claim.

## Command Lanes

| Surface | Status | Access path | Notes |
| --- | --- | --- | --- |
| `validate`, `plan`, `run`, `replay`, `verify`, `doctor`, `version` | stable | visible CLI | primary operator surface |
| `commands` | stable | visible CLI | route inventory for stable and non-stable command discovery |
| `capabilities` | internal | `BIJUX_DAG_ENABLE_INTERNAL=1` | maintainer-only support probe outside the public operator lane |
| `prove`, `export`, `import`, `migrate inspect` | experimental | explicit-path routes | supported with narrower expectations |
| control-plane, governance, incident, lab, federation, enterprise | simulated or internal | `commands --lane simulated` plus opt-in env for execution | repository proof and modeling surfaces |

`commands` defaults to the stable lane. The deliberate `--lane experimental`,
`--lane simulated`, and `--lane internal` views expose classified discovery;
they do not promote those commands into the stable operator promise.

## Execution Backends

| Surface | Status | Access path | Notes |
| --- | --- | --- | --- |
| local shell and container execution | stable | default `run` backend | host-process or local container execution under documented isolation limits |
| `run --backend slurm` on a shared filesystem | stable | visible `run` surface with explicit backend selection | submits through `sbatch`, polls `sacct`, and records retained batch evidence when the scheduled worker can reopen the same run directory |
| `run --backend kubernetes` for container nodes | stable | visible `run` surface with explicit backend selection | requires `--kubernetes-volume-claim`, `--kubernetes-shared-root`, and a shared persistent volume claim mounted into Job pods |
| Generic HPC beyond the shared-filesystem SLURM lane, public remote workers, full scheduler service | unreleased | not part of first-run adoption | broader portability and distributed control are not a `v0.4.0` product promise |

The stable backend rows describe supported orchestration, not stronger security
than the selected host, scheduler, or container engine provides. Read
[Execution Security And Isolation](../operations/security-isolation-truth.md)
before making an isolation claim.

## Verify The Boundary

- `cargo run -p bijux-dag-cli --bin bijux-dag -- commands`
- `BIJUX_DAG_ENABLE_INTERNAL=1 cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json`
- [Release Binary Verification](../../spec/RELEASE_BINARY_VERIFICATION.md)
- [Release Boundary](../foundation/release-boundary.md)
