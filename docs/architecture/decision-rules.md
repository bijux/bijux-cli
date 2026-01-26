# Decision rules

## Purpose
This document guarantees where decisions are made and where they are forbidden.

## Scope
This covers policy, exit behavior, and output routing.

## Core Concepts
- Policy resolution lives in core.
- Exit policy lives in core.
- Infra executes decisions, never makes them.

## Invariants
- No policy resolution outside `core/precedence.py`.
- No exit decisions outside `core/exit_policy.py`.
- No output routing outside core policy resolution.

## Execution
- Command handlers build payloads only.
- Emitters write exactly what policy dictates.

## Failure Modes
- Raw flag strings or ad-hoc routing are rejected by architecture tests.
- Policy changes outside core are considered defects.

## Design Rationale
- Alternatives: per-command decisions.
- Rejected because it creates inconsistent behavior.
- Chosen: centralized policy for determinism.

## Non-Goals
- CLI UX design guidance.

## References
- Enforcement tests: `tests/unit/core/test_architecture.py`
- Policy implementation: `src/bijux_cli/core/precedence.py`
