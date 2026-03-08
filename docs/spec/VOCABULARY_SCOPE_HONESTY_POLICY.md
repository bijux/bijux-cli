# Vocabulary and Scope Honesty Policy

## Objective

User-facing names must not imply stronger shipped capability than current evidence demonstrates.

## Rules

1. New user-facing terms must appear in `configs/policy/vocabulary_registry.json`.
2. Deprecated overreaching terms must map to canonical replacements.
3. CLI help and generated operator docs must use canonical terms.
4. New names implying production distributed/control-plane/auth-tenancy capability require explicit evidence links.

## Governance

- terminology consistency contracts: `crates/bijux-dev-dag/tests/vocabulary_scope_honesty_421_440_contracts.rs`
- terminology suite: `configs/suites/terminology_consistency_verification.json`
