# CLI usage

## Purpose
This document tells you how to run commands with predictable output.

## Scope
It covers usage patterns, not internal behavior.

## What problem this solves
Ad-hoc flags produce inconsistent output across machines.
This guide shows the stable patterns.

## Why you should care
Predictable output keeps scripts stable.

## What confusion this removes
It removes ambiguity about format, quiet, and log level.

## Guarantees
Bijux guarantees:
1. `--format` controls structured output.
2. `--quiet` never changes exit codes.

## How to Think About This
Treat output format as an explicit decision, not a default guess.

## Common Misunderstandings
- "Debug output can appear in JSON." It must not.

## Execution
Quick commands:

```bash
bijux --help
bijux --version
bijux doctor
```

Output formats:

```bash
bijux status --format json
bijux status --format yaml
```

Quiet mode:

```bash
bijux status --quiet
```

Log level:

```bash
bijux status --log-level debug
```

Shell completion:

```bash
bijux --install-completion
bijux --show-completion
```

## Failure Modes
- Invalid format exits with code 2.
- Invalid log level exits with code 2.

## Design Rationale
We deliberately chose explicit flags to avoid hidden behavior.
Why not auto-detect output? It breaks scripting consistency.

## Non-Goals
- Command-by-command reference.

## References
- Precedence rules: `concepts/precedence.md`
- Exit codes: `reference/exit-codes.md`
