# Logging

## Purpose
This document guarantees how log levels affect CLI output and diagnostics.

## Scope
This covers CLI logging behavior and routing only.

## Core Concepts
- Log level is part of the resolved policy.
- Diagnostics are emitted only when policy allows it.

## Invariants
- Quiet suppresses logs and output.
- Log level never changes exit codes.
- Structured output is not mixed with styled logs.

## Execution
- `info` is the default log level.
- `debug` and `trace` enable diagnostics and extra context.
- Diagnostics are routed to stderr.

## Failure Modes
- Invalid log level value: exit code 2.
- Unsupported configuration: exit code 2.

## Design Rationale
- Alternatives: ad-hoc logging per command.
- Rejected because it produces inconsistent diagnostics.
- Chosen: policy-driven logging.

## Non-Goals
- External log aggregation.
- Persistent log storage.

## References
- Implementation: `src/bijux_cli/core/precedence.py`
- Unit coverage: `tests/unit/cli/core/test_validation.py`
