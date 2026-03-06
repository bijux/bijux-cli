# Plugin and DSL roadmap

## Purpose

Define explicit extension points so adapter and backend integrations are intentional, typed, and maintainable.

## DSL roadmap

- Keep graph construction deterministic by design.
- Expand typed node/edge helper APIs without weakening core schema guarantees.
- Preserve stable compile and lint contracts as DSL evolves.

## Plugin roadmap

- Adapter plugins implement typed execution contracts.
- Backend plugins implement typed submission and completion contracts.
- Plugin manifests include name, version, type, and contract version.

## Guardrails

- Plugin behavior must emit standard observability events.
- Plugins must not bypass policy and capability checks.
- Plugin contracts must remain serialization-stable across supported versions.
