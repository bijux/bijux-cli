# Architecture

## Purpose
This section defines architectural rules and frozen guarantees.

## Scope
This covers decision boundaries and policy ownership only.

## Core Concepts
- Decision rules are binding constraints.
- Violations are treated as defects.

## Invariants
- Policy is resolved only in core.
- Exit behavior is resolved only in core.

## Execution
- Use decision rules when adding or refactoring modules.

## Failure Modes
- Boundary violations block review or break tests.

## Design Rationale
- Alternatives: permissive boundaries.
- Rejected because they cause policy drift.

## Non-Goals
- Feature documentation.

## References
- Decision rules: decision-rules.md
