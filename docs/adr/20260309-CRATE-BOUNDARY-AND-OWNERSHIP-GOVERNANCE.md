# Crate Boundary and Ownership Governance

Status: accepted
Owner: architecture maintainers
Date: 2026-03-09

## Decision
Each crate has explicit boundary, ownership, and dependency constraints. Cross-crate drift is governed by contract checks.

## Consolidated from
- 20260309-CRATE-BOUNDARY-GOVERNANCE.md
- 20260309-APP-CRATE-BOUNDARY.md
- 20260308-ARTIFACT-CRATE-SCOPE-RUNTIME-APP-BOUNDARIES.md
- 0010-APP-CRATE-STRUCTURE-DECISION.md

## Consequences
- Public API and dependency contracts are enforced by policy.
- New modules must align with declared ownership classes.
