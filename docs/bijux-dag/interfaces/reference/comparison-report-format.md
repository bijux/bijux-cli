---
title: Comparison Report Format
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Comparison Report Format

Use this format when publishing a comparison result derived from
`evidence/compare/`.

## Required sections

1. Scenario id
2. Scenario class
3. Target system
4. Measured `bijux` evidence asset
5. Baseline entry
6. Observed outcome
7. Non-equivalence limits
8. Release significance

## Example layout

```text
scenario_id: determinism
scenario_class: factual
target_system: orchestrators
bijux_evidence_asset: evidence/battle/workflows/happy_path/parse_validate_run_inspect_replay.json
baseline_entry: evidence/compare/baselines/bijux_v1.json#determinism
observed_outcome: completed without retry drift
non_equivalence_limits:
  - host clock scheduling jitter
  - external process nondeterminism
release_significance: advisory
```

## Authoring notes

- quote scenario ids exactly as they appear in `evidence/compare/scenarios/`
- keep interpretation separate from measured evidence
- cite the non-equivalence limits directly instead of implying global parity
