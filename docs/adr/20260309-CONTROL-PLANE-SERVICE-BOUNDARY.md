# Control Plane Service Boundary

Status: accepted
Owner: dev control-plane maintainers
Date: 2026-03-09

## Decision
`bijux-dev-dag` is the single control-plane automation surface for governance workflows. Root scripts and duplicated automation entrypoints are disallowed.

## Consolidated from
- 20260309-CONTROL-PLANE-SERVICE-MIGRATION-BOUNDARY.md
- 20260308-DEV-DAG-GOVERNANCE-SCOPE.md
- 20260308-DEV-DAG-COMMAND-DECOMPOSITION-SHAPE.md

## Consequences
- Governance automation remains centralized and auditable.
- Command-surface growth is controlled by freeze rules.
