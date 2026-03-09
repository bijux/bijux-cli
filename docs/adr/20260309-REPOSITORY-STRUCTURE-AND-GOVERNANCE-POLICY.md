# Repository Structure and Governance Policy

Status: accepted
Owner: repository maintainers
Date: 2026-03-09

## Decision
Repository structure is governed by clear boundaries for docs, control-plane automation, evidence, and runtime sources.

## Consequences
- Root and section sprawl are controlled by policy.
- Governance guards prevent unsupported structure drift.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-REPO-TREE-SHAPE-GOVERNANCE.md
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
- `crates/bijux-dev-dag/tests/repository_tree_simplification_contracts.rs`
