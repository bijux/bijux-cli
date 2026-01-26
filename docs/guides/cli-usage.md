# CLI usage

## Purpose
This document guarantees how to run CLI commands with predictable output.

## Scope
It covers usage patterns, not internal behavior.

## Core Concepts
- Output formats are explicit.
- Quiet suppresses output only.

## Invariants
- `--format` controls structured output.
- `--quiet` never changes exit codes.

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
- Alternatives: auto-detected formats.
- Rejected because they are non-deterministic.

## Non-Goals
- Command-by-command reference.

## References
- Precedence rules: `concepts/precedence.md`
- Exit codes: `reference/exit-codes.md`
