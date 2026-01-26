# Configuration

## Purpose
This document tells you how to set configuration for CI and scripts.

## Scope
It covers environment variables and config commands only.

## What problem this solves
Hidden config sources make troubleshooting slow.
This guide keeps config sources explicit.

## Why you should care
If config sources are explicit, you can trace a bad value to one place.

## What confusion this removes
It removes ambiguity about which source overrides another.

## Guarantees
Bijux guarantees:
1. Environment overrides config files.
2. CLI flags override everything.

## How to Think About This
Treat config as a stack of overrides with a fixed order.

## Common Misunderstandings
- "Config file overrides environment." It does not.

## Execution
Set defaults for CI:

```bash
export BIJUXCLI_FORMAT=json
export BIJUXCLI_LOG_LEVEL=info
```

Set config values:

```bash
bijux config set ci.enabled=true
bijux config get ci.enabled
```

## Failure Modes
- Invalid key syntax exits with code 2.
- Non-ASCII values exit with code 3.

## Design Rationale
We deliberately chose explicit sources to keep precedence clear.
Why not auto-discover configs? It hides overrides.

## Non-Goals
- Custom config file formats.

## References
- Config schema: `reference/config-schema.md`
- Precedence: `concepts/precedence.md`
