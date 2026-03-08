# Benchmark Scenario Contract

## Purpose
Define the canonical benchmark scenario registry for bijux-dag so benchmark claims always map to committed, versioned workloads.

## Required scenario fields
Every scenario JSON under `evidence/perf/scenarios/` must include:
- `scenario_id`
- `class`
- `size`
- `version`
- `owner`
- `graph`

## Registry authority
- Scenario registry file: `evidence/perf/scenario_registry.json`
- Performance metadata authority: `evidence/perf/metadata.json`
- Performance claim authority: `docs/spec/PERFORMANCE_CONTRACT.md`

## Required benchmark scenario identifiers
The registry must include the following scenario IDs:
- `tiny-canonical`
- `wide-canonical`
- `deep-canonical`
- `tenk-nodes-canonical`
- `large-artifact-canonical`
- `cache-heavy-canonical`
- `failure-injection-canonical`
- `replay-canonical`
- `diff-canonical`
- `portability-canonical`
- `determinism-score`
- `replay-fidelity-score`
- `explainability-quality`
- `artifact-lineage-completeness`
- `portability-success-rate`
- `inspect-history-latency`

## Evolution policy
- Changes to scenario meaning require `version` bump in the scenario file.
- Scenario removal must keep an archived reference in `evidence/inventory/benchmark_scenarios.md`.
