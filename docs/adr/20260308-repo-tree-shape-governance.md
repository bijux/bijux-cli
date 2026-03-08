# ADR: Repo Tree Shape Governance

## Status

Accepted

## Context

Repository structure drift and oversized modules increase maintenance cost and obscure ownership boundaries.

## Decision

1. Keep inventory reporting for size, churn, coverage, and ownership as stable governance outputs.
2. Track tiny-module inline candidates and giant-module split candidates continuously.
3. Require explicit split plans for new large files.
4. Keep repo-tree health and cleanup-candidate pages as always-on maintenance signals.

## Consequences

- Structural drift is caught earlier and tied to clear ownership actions.
- Large-file growth is constrained by explicit split planning.
- Cleanup opportunities remain visible and actionable.

## Enforcement

- `configs/policy/module_hygiene_governance.json`
- `configs/suites/repo_tree_simplification_verification.json`
- `crates/bijux-dev-dag/tests/repo_tree_simplification_541_560_contracts.rs`
