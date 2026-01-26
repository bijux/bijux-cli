# Architecture

## Purpose
This document guarantees the dependency direction and responsibility boundaries.

## Scope
This covers core, infra, services, and CLI layers only.

## Core Concepts
- Core owns policy, intent, and exit decisions.
- Infra provides concrete adapters only.
- Services compose infra into CLI-facing behavior.
- CLI builds intents and dispatches commands.

## Invariants
- Core never depends on infra or services.
- Infra never depends on services.
- Services depend on core and infra only.
- CLI depends on services only.

## Execution
- Intent is built in the CLI layer.
- Policy is resolved in core.
- Runtime is assembled from services and infra.

## Failure Modes
- Boundary violation is rejected in review or tests.
- Missing adapter is a runtime error with exit code 1.

## Design Rationale
- Alternatives: flat module graph.
- Rejected because it collapses boundaries and hides policy decisions.
- Chosen: strict layering with explicit ownership.

## Non-Goals
- Microservice decomposition.
- Cross-process orchestration.

## References
- Enforcement tests: `tests/unit/core/test_architecture.py`
