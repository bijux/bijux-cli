# App Routing and Service Boundary Policy

Status: accepted
Owner: app maintainers
Date: 2026-03-09

## Decision
The app crate routes command families through dedicated route modules with explicit service boundaries and ownership clarity.

## Consequences
- Router responsibilities remain explicit and testable.
- Business logic residue in route modules is treated as drift.

## Merged Decision Record
This ADR is standalone. The historical decision text merged into this record is included below.

### SOURCE: 20260308-APP-ROUTER-FINAL-END-STATE.md
# ADR: App Router Final End-State

Date: 2026-03-08

## Context

`crates/bijux-dag-app/src/lib.rs` accumulated command branching, JSON assembly, and
human-output rendering over time. This made routing difficult to review and blocked
module-level coverage goals.

## Decision

`lib.rs` remains the top-level command dispatcher and shared utility host only.
Command-family behavior is delegated to route modules:

- inspect routes in `routes/inspect_routes.rs`
- plan routes in `routes/plan_routes.rs`
- diagnostics routes in `routes/diagnostics_routes.rs`
- surface/capability routes in `routes/surface_routes.rs`

Router-specific policy constraints:

- file-size ceilings for `routes/inspect_routes.rs` and `routes/plan_routes.rs`
- explicit route coverage targets for inspect, plan, diagnostics, output-selection, and surface routes
- contract checks that key command families delegate through route modules

## Consequences

- routing changes are local to route modules
- response and rendering logic are no longer expanded in top-level dispatch branches
- router decomposition can be measured and enforced in CI through contract tests and policy files
