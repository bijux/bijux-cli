# ADR: Runtime Scope End State

## Status

Accepted

## Context

Runtime surfaces expanded beyond deterministic execution needs, creating ambiguity between shipped capability and modeled or incubating behavior.

## Decision

1. Runtime modules are classified as `core`, `adapter`, `operator-support`, `experimental`, or `speculative`.
2. `experimental` and `speculative` modules remain quarantined from default capability/help/operator surfaces.
3. Every new runtime module must declare lifecycle status in governance policy before merge.
4. `experimental` and `speculative` modules require explicit expiration criteria.

## Consequences

- Runtime scope remains bounded to deterministic execution by default.
- Broad or modeled surfaces are documented without overstating stable capability.
- Scope drift becomes detectable and merge-blocking via contract suites.

## Enforcement

- `configs/policy/runtime_module_lifecycle_status.json`
- `configs/suites/runtime_scope_contraction_verification.json`
- `crates/bijux-dev-dag/tests/runtime_scope_contraction_guarantees_contracts.rs`
