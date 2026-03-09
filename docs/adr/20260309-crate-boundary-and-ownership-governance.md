# Crate Boundary and Ownership Governance

Status: accepted
Owner: architecture maintainers
Date: 2026-03-09

## Decision
Each crate has explicit boundary, ownership, and dependency constraints. Cross-crate drift is governed by contract checks.

## Consolidated from
- 20260309-crate-boundary-governance.md
- 20260309-app-crate-boundary.md
- 20260308-artifact-crate-scope-runtime-app-boundaries.md
- 0010-app-crate-structure-decision.md

## Consequences
- Public API and dependency contracts are enforced by policy.
- New modules must align with declared ownership classes.
