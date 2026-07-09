# Evidence Status Memo

## Authoritative Today

- `evidence/_meta/registries/evidence_registry.json` is the canonical evidence registry.
- `evidence/ownership/evidence_ledger.json` is machine-normalized and ownership-governed.
- `evidence/release/release_evidence_set.json` defines blocking and advisory release evidence.
- `configs/dag/schema/evidence_*.schema.json` define metadata and family schema contracts.
- `configs/dag/policy/evidence_suite_policy.json` defines evidence verification suite enforcement modes.

## Enforced Governance Boundaries

- Evidence assets are validated through schema, registry, ownership, consumer, and drift checks.
- Release readiness is evidence-driven and cannot rely on aggregate test counts alone.
- Advisory evidence cannot be reported as blocking release proof.
- Legacy roots (`examples/`, `benchmarks/`, `comparisons/`) must remain deleted as canonical sources.

## Current Follow-Through Items

- Reduce remaining perf and compare assets that do not add durable release value.
- Continue pruning stale assets flagged by evidence reports.
- Keep battle coverage focused on trust-property protection without overloaded scenarios.

## Architecture Freeze Rule

Evidence family model, registry shape, and control-plane verification surfaces are frozen by default.
Any structural expansion requires a contract update, registry support, and a new verify surface.
