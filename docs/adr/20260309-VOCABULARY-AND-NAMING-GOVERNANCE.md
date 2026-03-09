# Vocabulary and Naming Governance

Status: accepted
Owner: naming maintainers
Date: 2026-03-09

## Decision
Normative vocabulary and naming are stability contracts. Transitional and ambiguous terms are excluded from durable surfaces.

## Consequences
- Naming reviews are required for new normative terms.
- Contract language remains precise and long-lived.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-VOCABULARY-AND-SCOPE-HONESTY.md
# ADR: Vocabulary and Scope Honesty

## Status

Accepted

## Context

Repository and runtime surfaces include modeled and speculative modules whose names can imply broader shipped capability than current product guarantees.

## Decision

1. Maintain a canonical/deprecated vocabulary registry with explicit replacements.
2. Enforce terminology consistency in user-facing help and generated documentation.
3. Track stale references in docs/tests/examples/evidence outputs for controlled migration.
4. Reject new overreaching names without evidence-backed scope justification.

## Consequences

- Product wording becomes explicit about shipped vs modeled scope.
- Operator confusion and expectation drift are reduced.
- Naming drift is detected via testable governance artifacts.

## Enforcement

- `configs/policy/vocabulary_registry.json`
- `docs/spec/VOCABULARY_SCOPE_HONESTY_POLICY.md`
- `configs/suites/terminology_consistency_verification.json`
- `crates/bijux-dev-dag/tests/vocabulary_scope_honesty_guarantees_contracts.rs`
