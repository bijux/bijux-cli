# Test Trust Ledger

## Scope

This contract defines the policy ledger that classifies runtime test files,
marks must-never-break surfaces, and records the semantic surfaces the cleanup
guard expects to stay explicit.

## Authoritative input

`configs/dag/policy/test_trust_ledger.json` is the authoritative machine
source for classification rules, must-never-break coverage, and trust-family
maintenance decisions.

## Required policy sections

- `classification_rules`
- `must_never_break`
- `required_semantic_surfaces`
- `snapshot_surface_policy`
- `trust_coverage_families`

## Versioning and change policy

Ledger section names and cleanup intent are stable contract surfaces. Any
incompatible change requires updating this document, the ledger policy, and the
maintenance report in the same change.
