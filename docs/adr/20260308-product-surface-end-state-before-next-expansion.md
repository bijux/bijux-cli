# ADR: Product Surface End-State Before Next Expansion

- Date: 2026-03-08
- Status: Accepted

## Context
The 1-600 engineering program hardened correctness, portability, and surface stability while exposing persistent sources of awkwardness in command structure, runtime shape, repository ergonomics, and governance sprawl.

## Decision
Before any next expansion, the system must hold this end-state:

1. Stable execution truth remains anchored in deterministic local semantics.
2. Operator-facing commands are canonical, concise by default, and schema-stable.
3. Runtime public surface is minimal; speculative and experimental scopes remain quarantined.
4. Evidence, benchmark, and fixture families remain decision-value-driven and non-duplicative.
5. Internal contracts have owners, direct tests, fixture links, and specification links.
6. Dashboards and reports remain audience-specific and aligned to blocker paths.

## Consequences
- New scope can only enter through explicit ownership, direct verification, and vocabulary honesty checks.
- Existing awkwardness sources are prioritized for removal before broadening product claims.
- Engineering velocity improves through smaller canonical surfaces and lower maintenance drag.
