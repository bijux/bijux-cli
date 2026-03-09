# Runtime Scope and Quarantine Policy

Status: accepted
Owner: runtime maintainers
Date: 2026-03-09

## Decision
Runtime core stays focused on deterministic execution and stable APIs. Experimental and speculative semantics remain quarantined and cannot leak into default operator or identity surfaces.

## Consolidated from
- 20260309-RUNTIME-CONTRACTION-GOVERNANCE.md
- 20260309-RUNTIME-QUARANTINE-RATIONALE.md
- 20260308-ADVANCED-SEMANTICS-RUNTIME-BOUNDARY.md

## Consequences
- Runtime public API remains minimal and durable.
- Quarantined modules require explicit lifecycle and graduation criteria.
