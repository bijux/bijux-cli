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

## Logging Semantics (Truth Table)

Flags resolve in this order: `quiet` → `debug` → `verbose` → `log_level`.
Color defaults to `auto` and is only overridden by `--color`.

| quiet | debug | verbose | log_level flag | effective log_level | include runtime | pretty |
|------:|------:|--------:|---------------:|--------------------:|----------------:|-------:|
| false | false | false   | info           | info                | false           | flag   |
| false | false | true    | info           | info                | true            | flag   |
| false | true  | any     | any            | debug               | true            | true   |
| true  | any   | any     | any            | error               | false           | flag   |

## Serializer Rules

- Serializer formats output; CLI never formats directly.
- Services choose representation (JSON/YAML), not the CLI.
- JSON output is colorless and stable for automation.
