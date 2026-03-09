# App Routing and Service Boundary Policy

Status: accepted
Owner: app maintainers
Date: 2026-03-09

## Decision
The app crate routes command families through dedicated route modules with explicit service boundaries and ownership clarity.

## Consolidated from
- 20260308-app-router-final-end-state.md

## Consequences
- Router responsibilities remain explicit and testable.
- Business logic residue in route modules is treated as drift.
