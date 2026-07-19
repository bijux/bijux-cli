---
title: Comparison Report Format
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Comparison Report Format

Use this page to interpret or publish evidence from the repository comparison
harness. The harness does not define one self-contained comparison-result file.
It joins a scenario record, registry metadata, a Bijux evidence asset, and a
versioned baseline. Dropping any of those inputs weakens the claim.

## Authority Chain

| Surface | Authority |
| --- | --- |
| `evidence/compare/scenarios/` | one record per stable scenario ID, with description, workload shape, and focus |
| `evidence/compare/metadata.json` | scenario class, target system, measured Bijux asset, equivalence note, limits, and release role |
| referenced `bijux_evidence_asset` | direct evidence for the Bijux behavior under discussion |
| `evidence/compare/baselines/bijux_v1.json` | expected scenario status, retry count, and cache-hit count for the governed baseline |
| `docs/spec/COMPARISON_HARNESS_CONTRACT.md` | minimum evidence set and interpretation rules |
| `crates/bijux-dev/src/commands/compare_evidence.rs` | validation and report generation |

Paths beginning with `evidence/compare/` resolve through the repository's
governed comparison evidence root. The canonical files are retained under
`evidence/dag/compare/`; the root-level path is a repository compatibility
link, not a second source of truth.

## Scenario Record

A scenario file identifies the workload. It does not carry release policy or a
result:

```json
{
  "id": "determinism",
  "description": "Repeated runs with fixed inputs produce stable outcomes",
  "shape": "medium",
  "focus": ["determinism", "replay"]
}
```

Scenario IDs must be unique and must match a baseline entry. Changing an ID is
an evidence migration because tests and historical comparisons address it
directly.

## Registry Entry

The metadata registry supplies the claim boundary for each scenario path:

```json
{
  "target_system": "orchestrators",
  "scenario_class": "factual",
  "bijux_evidence_asset": "evidence/battle/workflows/happy_path/parse_validate_run_inspect_replay.json",
  "scenario_equivalence_note": "deterministic behavior claims",
  "non_equivalence_limits": [
    "host clock scheduling jitter",
    "external process nondeterminism"
  ],
  "limitation_note": "environment-level nondeterminism excluded",
  "release_blocking": false,
  "measured_bijux_side": true
}
```

The machine schema is
`configs/dag/schema/evidence_compare_metadata.schema.json`. Each registry entry
must name a real scenario file and real Bijux evidence asset. Every scenario
must declare at least one non-equivalence limit.

`scenario_class` has two meanings:

- `factual` permits a bounded statement about directly measured Bijux behavior;
- `descriptive` permits context and interpretation but cannot serve as release
  evidence.

A release-blocking entry is valid only when the Bijux side was measured. The
current registry marks its scenarios non-blocking, so a successful comparison
report is advisory evidence rather than release approval.

## Baseline Entry

The baseline uses `comparison-baseline/v1` and records one row per scenario:

```json
{
  "id": "retry-timeout",
  "status": "failed",
  "retries": 2,
  "cache_hits": 0
}
```

`status: failed` can be the expected outcome of a failure scenario. It does not
mean the comparison harness failed. Interpret status together with the
scenario description, source evidence, retry count, and declared limits.

The baseline is a governed regression reference for this repository version.
It is not an external benchmark, a ranking, or timeless proof about another
orchestrator.

## Generated Report

Run the maintainer producer from the repository root:

```bash
bijux-dev-dag comparison-evidence-report
```

Its JSON payload contains:

- the registry purpose;
- `factual` rows with scenario path, target system, Bijux evidence asset,
  non-equivalence limits, and release-blocking state;
- `interpretation_only` rows for descriptive scenarios;
- the number of baseline entries;
- the retained fact-versus-interpretation report path.

The producer validates metadata before printing. It rejects missing scenarios,
missing evidence assets, invalid classes, empty limits, insufficient factual
coverage, and release-blocking scenarios without a measured Bijux side.

The command does not execute every workload or mint a fresh benchmark result.
Its output is an inventory and policy report over retained evidence. Do not
label its process exit alone as a new experimental measurement.

## Publication Requirements

A human-facing comparison must include:

1. the exact scenario path and scenario ID;
2. the metadata registry version;
3. the scenario class and target system;
4. the referenced Bijux evidence asset;
5. the matching baseline row when baseline behavior is discussed;
6. every non-equivalence limit and the limitation note;
7. the release-blocking and measured-side flags;
8. the source commit or release that fixes these inputs.

Keep observed facts separate from interpretation. Do not infer implementation
equivalence from output similarity, omit a failed baseline row because it looks
unfavorable, or convert advisory evidence into a release gate.

## Verification

```bash
cargo test -p bijux-dag-app --test comparison_harness_contract
cargo run --locked -p bijux-dev --bin bijux-dev-dag -- comparison-evidence-report
```

The focused test proves unique scenario IDs and baseline coverage. The
maintainer command validates registry policy and renders the current inventory.
Neither command supports a broad superiority claim.

## Continue Reading

- [Comparison Evidence Surfaces](../quality/comparison-evidence-surfaces.md)
- [Reproducibility Model](reproducibility-model.md)
- [Known Limitations](../quality/known-limitations.md)
