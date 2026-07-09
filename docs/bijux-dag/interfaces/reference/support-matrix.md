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

The contract source for this matrix is the
[Release Boundary](../../foundation/release-boundary.md) page, which is backed
by the machine-readable contract
`contracts/foundation/dag_release_truth_table.v1.json`.
For the longer release ladder beyond the current matrix, use the
[Bijux Dag Roadmap](../../../tracking/bijux-dag-roadmap.md).

| Surface | Status | Access path | Notes |
| --- | --- | --- | --- |
| `validate`, `plan`, `run`, `replay`, `verify`, `doctor`, `version` | stable | visible CLI | primary operator surface |
| `commands` | stable | visible CLI | route inventory for the stable surface plus deliberate `--lane experimental`, `--lane simulated`, and `--lane internal` discovery |
| `capabilities` | internal | `BIJUX_DAG_ENABLE_INTERNAL=1` | maintainer-only support probe outside the public operator lane |
| `prove`, `export`, `import`, `migrate inspect` | experimental | explicit-path routes | supported with narrower expectations |
| control-plane, governance, incident, lab, federation, enterprise | simulated or internal | `commands --lane simulated` plus opt-in env for execution | repository proof and modeling surfaces |
| `run --backend slurm` on a shared filesystem | stable | visible `run` surface with explicit backend selection | submits through `sbatch`, polls `sacct`, and records retained batch evidence when the scheduled worker can reopen the same run directory |
| `run --backend kubernetes` for container nodes | stable | visible `run` surface with explicit backend selection | requires `--kubernetes-volume-claim`, `--kubernetes-shared-root`, and a shared persistent volume claim mounted into Job pods |
| Generic HPC beyond the shared-filesystem SLURM lane, public remote workers, full scheduler service | unreleased | not part of first-hour adoption | broader portability and distributed control are not a `v0.4.0` product promise |

## Primary proof

- `cargo run -p bijux-dag-cli --bin bijux-dag -- commands`
- `BIJUX_DAG_ENABLE_INTERNAL=1 cargo run -p bijux-dag-cli --bin bijux-dag -- capabilities --json`
- `docs/spec/RELEASE_BINARY_VERIFICATION.md`
- `docs/bijux-dag/foundation/release-boundary.md`

## Next Reads

- [Release Boundary](../../foundation/release-boundary.md)
- [Known Limitations](../../quality/known-limitations.md)
- [Bijux Dag Roadmap](../../../tracking/bijux-dag-roadmap.md)
