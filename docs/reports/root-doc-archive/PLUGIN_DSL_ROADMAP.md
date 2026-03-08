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
- Artifact store plugins implement conformance-verified storage contracts.
- Observability plugins implement typed sink/export contracts.
- Plugin manifests include name, version, type, and contract version.

## Trust and release policy

- Official plugins require core-team review, security assessment, and conformance pass.
- Community plugins remain supported through stable contracts but are not default-distributed.
- Plugin loading is static-link-first; dynamic loading is deferred until trust/isolation controls are mature.

## Lifecycle model

- develop
- register
- validate
- release
- deprecate
- remove

## Discovery and diagnostics

- Extension registration records must be visible in manifests and diagnostics.
- Operators can build extension discovery inventory from registration records.

## DSL extension points

- Custom node families must use compile-time validated extension points.
- Extensions may not bypass graph compile validation or deterministic rules.
- Code generation hooks may emit schema and task-contract bindings.

## Guardrails

- Plugin behavior must emit standard observability events.
- Plugins must not bypass policy and capability checks.
- Plugin contracts must remain serialization-stable across supported versions.
