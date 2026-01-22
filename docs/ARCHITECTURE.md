# Architecture Rules

Dependency direction is strict and non-negotiable:

- core → no infra, no services
- infra → no core, no services
- services → core + infra
- cli → services only
- app → wires everything

These rules prevent architectural drift and keep boundaries testable.

## Contract Ownership Rules

- core: cross-cutting behavior and infra-facing interfaces only
- services: service protocols and expectations (config/diagnostics/history/etc.)
- infra: concrete adapters that implement core interfaces

## Plugin Pipeline

Stages are fixed and ordered:

1. discover
2. validate metadata
3. register
4. activate (lazy)
5. unload (if applicable)

Enforce this order in code and reviews.

## v0.2 Architecture Freeze

Until v0.2.0 ships:

- no new layers
- no new cross-dependencies
- only bug fixes and plugin hardening

## Output Precedence (Truth Table)

Flags resolve in this order: `quiet` → `log_level` → `verbose` → defaults.
`--json` forces JSON output; color settings do not affect JSON payloads.

| quiet | log-level flag | -v/-vv | --json | --color | effective log_level | include runtime | format | color |
|------:|----------------|-------:|-------:|--------:|--------------------:|----------------:|-------:|------:|
| false | info           | 0      | false  | auto    | info                | false           | json   | auto  |
| false | info           | 1      | false  | auto    | info                | true            | json   | auto  |
| false | warning        | 2      | false  | always  | warning             | true            | json   | always|
| false | error          | 0      | true   | never   | error               | false           | json   | never |
| true  | any            | any    | any    | any     | error               | false           | json   | any   |

## Serializer Rules

- Serializer formats output; CLI never formats directly.
- Services choose representation (JSON/YAML), not the CLI.
- JSON output is colorless and stable for automation.
