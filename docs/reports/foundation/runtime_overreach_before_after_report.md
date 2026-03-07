# Runtime Overreach Before/After Report

## Purpose

Track reduction of speculative runtime breadth so runtime remains a foundation DAG engine.

## Before (current inventory)

- Overreach modules identified in policy: 12
- `move`: 11
- `retain` (foundation-critical): 1 (`artifacts/storage/semantic_lineage.rs`)

## After (this pass)

- Immediate code deletion/move count: 0 (non-breaking governance pass)
- New enforced policy: `configs/policy/runtime_overreach_cleanup.json`
- New enforcement contract: `crates/bijux-dev-dag/tests/runtime_overreach_contracts.rs`
- Release-evidence dependency on overreach modules: blocked by contract checks

## Decisions by surface

| Surface family | Decision |
| --- | --- |
| AI operator assist | move |
| Workflow productization | move |
| Ecosystem packaging/adoption | move |
| Dataset semantics | move |
| Cost optimization | move |
| Adaptive scheduler | move |
| Federated scheduling | move |
| Geo federation | move |
| HA scheduler | move |
| Control-plane API in runtime | move |
| Provenance compliance policy | move |
| Semantic lineage storage integrity | retain |

## Next reduction step

Move `move` surfaces behind non-runtime ownership in dedicated migration commits while keeping runtime contracts green.
