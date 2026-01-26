# Decision rules

## Purpose
This document tells you where decisions are made and where they are forbidden.

## Scope
It covers policy, exit behavior, and output routing.

## What problem this solves
Policy decisions hidden in commands cause inconsistent behavior.
These rules prevent that drift.

## Why you should care
If you violate these rules, you break tests and user guarantees.

## What confusion this removes
It removes ambiguity about which module owns each decision.

## Guarantees
Bijux guarantees:
1. No policy resolution outside `core/precedence.py`.
2. No exit decisions outside `core/exit_policy.py`.
3. No output routing outside core policy resolution.

## How to Think About This
Treat policy as a single decision point. Everything else is execution.

## Common Misunderstandings
- "Commands can override routing." They cannot.

## Execution
- Command handlers build payloads only.
- Emitters write exactly what policy dictates.

## Failure Modes
- Raw flag strings or ad-hoc routing fail architecture tests.

## Design Rationale
We deliberately chose centralized policy to keep behavior deterministic.
Why not let each command decide? It creates divergence and hard-to-find bugs.

## Non-Goals
- CLI UX design guidance.

## References
- Enforcement tests: `tests/unit/core/test_architecture.py`
- Policy implementation: `src/bijux_cli/core/precedence.py`
