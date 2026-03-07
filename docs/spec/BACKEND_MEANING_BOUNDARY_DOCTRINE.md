# Backend Meaning Boundary Doctrine

## Rule

Backends execute semantics; they must not redefine graph/run/artifact meaning.

## Invariants

- Canonical identity and lineage rules live in core/runtime contracts, not backend-specific reinterpretation.
- Backend capability gaps must be surfaced as capability states, not silent semantic drift.
- Unsupported backends must remain clearly labeled as modeled/simulated/unsupported.

## Enforcement

- Support policy in `docs/reference/EXECUTION_SUPPORT_POLICY.md`.
- Runtime and evidence conformance suites in `bijux-dev-dag`.
