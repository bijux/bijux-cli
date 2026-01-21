# Architecture Rules

Dependency direction is strict and non-negotiable:

- core → no infra, no services
- infra → no core, no services
- services → core + infra
- cli → services only
- app → wires everything

These rules prevent architectural drift and keep boundaries testable.

## Plugin Pipeline

Stages are fixed and ordered:

1. discover
2. validate metadata
3. register
4. activate (lazy)

Enforce this order in code and reviews.
