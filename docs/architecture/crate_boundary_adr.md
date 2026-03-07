# Crate Boundary ADR

## Decision
Keep current crate set and enforce boundary contracts through `bijux-dev-dag` governance suites.

## Accepted boundaries
- runtime does not depend on app or cli crates
- app does not host scheduler/adapter execution internals
- cli remains thin and delegates command behavior to app
- artifact models and storage contracts remain in artifacts crate

## Service interface rule
- runtime exposes typed service interfaces for execution and artifact persistence boundaries
- artifacts exposes persistence service interfaces for run-dir operations

## Non-decisions
- No crate collapse at this time.
- No public stable runtime embedding API commitment beyond documented surfaces.

## Consequences
- Feature work must update boundary docs, tests, and governance checks in the same change.
- New dependency edges require explicit policy update and ownership rationale.
