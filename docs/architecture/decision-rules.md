# Decision Rules

## Purpose
This document defines the architectural rules that preserve bijux-cli invariants. It exists to prevent accidental design drift as the codebase evolves.

## Scope
It covers decision ownership and forbidden behaviors. It does not specify implementation details or coding style.

## Core Rules
Decisions about policy, output routing, and exit behavior must occur in core. Infra components must execute explicit instructions and never guess or normalize values.

## Why These Rules Exist
These rules prevent late overrides and inconsistent behavior across different entry points. They are enforced by tests and are considered part of the public contract.

## Failure Modes
When rules are violated, behavior becomes nondeterministic and tests may pass locally while failing in automation. These violations are treated as regressions and must be corrected, not documented away.

## Design Rationale
By centralizing decisions, we make the system predictable and testable. This also makes it easier to reason about failure handling and exit codes.

## Non-Goals
This document does not define project process or governance.
