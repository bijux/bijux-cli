# Exit policy

## Purpose
This document tells you which exit code and output stream a failure uses.

## Scope
It covers CLI exit codes and error routing only.

## What problem this solves
If exit codes shift, scripts break. This policy prevents that drift.

## Why you should care
You can build automation that depends on stable numeric outcomes.

## What confusion this removes
It removes doubt about whether output format or quiet mode changes exit codes.

## Guarantees
Bijux guarantees:
1. Each error class maps to a stable exit code.
2. Quiet suppresses output but never changes exit codes.
3. Output format never changes exit codes.

## How to Think About This
Treat exit codes as part of the interface, not an implementation detail.
If you need a different exit behavior, change policy in one place.

## Common Misunderstandings
- "Quiet mode hides errors by changing exit codes." It does not.
- "JSON output uses different exit codes." It does not.

## Execution
- Success exits with code 0.
- Usage or user input errors exit with code 2.
- ASCII or encoding errors exit with code 3.
- Internal errors exit with code 1.
- User abort exits with code 130.

## Failure Modes
- Invalid command: exit 2 with structured error.
- Internal exception: exit 1 with structured error.
- Encoding failure: exit 3 with structured error.

No retries occur at the exit policy layer.

## Design Rationale
We deliberately chose a centralized policy to keep exit codes stable.
Why not let each command choose? That creates inconsistent scripting behavior.

## Non-Goals
- Retrying failed commands.
- Plugin-specific exit codes.

## References
- Implementation: `src/bijux_cli/core/exit_policy.py`
- Regression coverage: `tests/regression/test_exit_policy.py`
