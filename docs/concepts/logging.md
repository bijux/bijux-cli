# Logging

## Purpose
This document tells you how log level affects output and diagnostics.

## Scope
It covers CLI logging only.

## What problem this solves
If logs leak into structured output, automation breaks. This policy prevents it.

## Why you should care
You can enable diagnostics without corrupting machine-readable output.

## What confusion this removes
It removes ambiguity about where logs go and when they appear.

## Guarantees
Bijux guarantees:
1. Quiet suppresses logs and output.
2. Log level never changes exit codes.
3. Structured output is never mixed with styled logs.

## How to Think About This
Logs are a separate stream with a separate purpose.
They explain behavior, but they never alter command results.

## Common Misunderstandings
- "Debug logs can appear in JSON output." They must not.
- "Log level changes exit codes." It does not.

## Execution
- `info` is the default.
- `debug` and `trace` emit diagnostics to stderr.

## Failure Modes
- Invalid log level exits with code 2.

## Design Rationale
We deliberately chose policy-driven logging to keep output stable.
Why not log inside commands? It leaks logs into structured output.

## Non-Goals
- Persistent log storage.

## References
- Implementation: `src/bijux_cli/core/precedence.py`
- Unit coverage: `tests/unit/cli/core/test_validation.py`
