# Exit policy

## Purpose
This document guarantees exit codes and output routing for failures.

## Scope
This covers CLI exit codes, stdout/stderr routing, and structured error payloads.

## Core Concepts
- ExitIntent encodes the final exit decision.
- Routing is decided once in core policy resolution.

## Invariants
- A given error class always maps to a stable exit code.
- Quiet mode suppresses output but never changes the exit code.
- Output format never changes the exit code.

## Execution
- Success exits with code 0.
- User input or usage errors exit with code 2.
- ASCII or encoding errors exit with code 3.
- Internal errors exit with code 1.
- User abort exits with code 130.

## Failure Modes
- Invalid command: exit 2 with structured error.
- Internal exception: exit 1 with structured error.
- Encoding failure: exit 3 with structured error.

No retries occur at the exit policy layer.

## Design Rationale
- Alternatives: command-level exit logic.
- Rejected because it diverges across commands.
- Chosen: centralized exit policy for stable scripting behavior.

## Non-Goals
- Retrying failed commands.
- Plugin-specific exit codes.

## References
- Implementation: `src/bijux_cli/core/exit_policy.py`
- Regression coverage: `tests/regression/test_exit_policy.py`
