# Evidence Publication Contract

## Purpose
Define publication rules for benchmark evidence used in docs, release notes, and comparisons.
Canonical terms are defined in `docs/spec/EVIDENCE_GLOSSARY.md`.

## Evidence publication rules
- Performance claims in docs must point to committed benchmark artifacts.
- Published evidence must include both raw benchmark report data and scenario metadata.
- Generated summaries may be used only when raw reports are still available.
- Evidence with missing scenario registry links is non-compliant.

## Required publication surfaces
- Scenario definitions: `evidence/perf/scenarios/`
- Scenario registry: `evidence/perf/scenario_registry.json`
- Metadata policy: `evidence/perf/metadata.json`
- Baselines and thresholds: `evidence/perf/baselines/`

## Governance
- `docs/spec/PERFORMANCE_CONTRACT.md` remains the top-level claim policy.
- This contract governs publication quality and traceability for benchmark evidence.
