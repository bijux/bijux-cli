---
title: Cache Correctness Coverage
audience: maintainer
type: report
status: active
owner: bijux-dag-maintainers
last_reviewed: 2026-07-06
---

# Cache Correctness Coverage

This ledger records the current governed cache correctness surfaces.

## Covered surfaces

- cache proof field completeness
- cache metadata version acceptance and refusal
- cache entry manifest version acceptance and refusal
- corruption fixtures from `evidence/cache/corrupt/`
- warm/cold semantic equivalence from `evidence/cache/scenarios/warm_cold.json`
- operator command coverage for explain, stats, prune-simulate, diff, and
  verify

## Maintenance rule

- add new cache proof or corruption behavior here when it becomes a governed
  surface
- remove an entry only in the same change that removes its proof obligation
