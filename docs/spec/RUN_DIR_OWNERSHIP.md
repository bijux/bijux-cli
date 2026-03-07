# Run Directory Ownership

## Purpose
Define ownership for each persisted run artifact so write/read responsibility is explicit.

## Ownership table

| Artifact path | Authoritative | Owner module |
| --- | --- | --- |
| `manifest.json` | yes | `bijux-dag-artifacts::RunDir` |
| `graph.snapshot.json` | yes | `bijux-dag-artifacts::RunDir` |
| `nodes/<node_id>/trace.json` | yes | `bijux-dag-artifacts::RunDir` + runtime engine writer |
| `outputs/index.json` | yes | `bijux-dag-artifacts::RunDir` + runtime engine writer |
| `provenance.json` | no | app import/export surface |
| `latest` symlink | no | app run lifecycle surface |

## Rules
- Only owner modules may define path conventions for authoritative run artifacts.
- New authoritative files require contract update in `RUN_DIR_CONTRACT.md` and this table.
- Derived files must never override or shadow authoritative files.
