# Architecture

## Purpose
This document tells you which module is allowed to make which decision.

## Scope
It covers core, infra, services, and CLI boundaries.

## What problem this solves
Without strict boundaries, policy decisions leak into random files.
That makes behavior inconsistent and hard to test.

## Why you should care
If you touch a boundary, you can break guarantees across the CLI.
This document tells you where that line is.

## What confusion this removes
It removes doubt about which module owns policy and exit behavior.

## Guarantees
Bijux guarantees:
1. Core never depends on infra or services.
2. Infra never depends on services.
3. Services depend on core and infra only.
4. CLI depends on services only.

## How to Think About This
Treat boundaries as enforcement points, not suggestions.
If a decision crosses the line, it is a defect.

## Common Misunderstandings
- "Infra can pick defaults." It cannot.
- "CLI can decide output routing." It cannot.

## Execution
- Intent is built in the CLI command layer.
- Policy is resolved in core.
- Runtime is assembled from services and infra.

## Failure Modes
- Boundary violations are rejected by architecture tests.
- Missing adapters fail at runtime with exit code 1.

## Design Rationale
We deliberately chose strict boundaries to keep decisions centralized.
Why not a flat module graph? It hides decisions and breaks tests.

## Non-Goals
- Cross-process orchestration.

## References
- Enforcement tests: `tests/unit/core/test_architecture.py`
