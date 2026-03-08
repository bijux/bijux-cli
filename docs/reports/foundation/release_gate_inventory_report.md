# Release Gate Inventory Report

| Gate | Owner | Purpose |
| --- | --- | --- |
| `fmt` | `dag-control-plane` | formatting consistency |
| `lint` | `dag-control-plane` | lint and static quality enforcement |
| `audit` | `dag-control-plane` | dependency and supply-chain checks |
| `test` | `dag-control-plane` | fast test lane signal |
| `test-all` | `dag-control-plane` | full test lane signal |
| `coverage` | `dag-control-plane` | line and contract coverage verification |
| `evidence-all` | `dag-control-plane` | evidence governance and release-evidence checks |

Source: `configs/policy/release_gate_governance.json`
