# Configuration

## Purpose
This document guarantees how to configure bijux-cli for CI and scripts.

## Scope
It covers environment variables and config commands only.

## Core Concepts
- Environment variables override config files.
- Config keys are stored in a dotenv-style file.

## Invariants
- `BIJUXCLI_` prefix is required for stored keys.
- Explicit CLI flags override all config sources.

## Execution
Set defaults for CI:

```bash
export BIJUXCLI_FORMAT=json
export BIJUXCLI_LOG_LEVEL=info
```

Set config values:

```bash
bijux config set ci.enabled true
bijux config get ci.enabled
```

## Failure Modes
- Invalid key syntax exits with code 2.
- Non-ASCII values exit with code 3.

## Design Rationale
- Alternatives: implicit config discovery.
- Rejected because it hides precedence.

## Non-Goals
- Custom config file formats.

## References
- Config schema: `reference/config-schema.md`
- Precedence: `concepts/precedence.md`
