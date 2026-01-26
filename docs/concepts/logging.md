# Logging

## Purpose
This document explains the logging semantics of bijux-cli. It defines log levels, routing, and how logging relates to output formatting.

## Scope
It covers log level selection and the relationship between logs and command output. It does not define the internal logging implementation details.

## Core Concepts
Logging is diagnostic, not output. Logs follow the configured log level and are routed separately from structured command output. Output format flags control command responses, not logs.

## How to Think About This
Think of logs as a parallel channel for operators. Structured output is for machines and should remain stable regardless of logging settings. If you need stable automation, parse structured output and treat logs as optional context.

## Invariants
- Output format does not change logging behavior.
- Log level controls verbosity but does not change command results.
- Logs must never override or mutate structured outputs.

## Failure Modes
If logging is misconfigured, the CLI still emits command output correctly. Logging failures must not prevent command execution or alter exit codes.

## Design Rationale
Separating logs from output preserves machine-readable behavior and prevents accidental coupling between diagnostics and results.

## Non-Goals
This document does not specify logging backends or third-party logging integration.

## References
- Implementation: `src/bijux_cli/services/diagnostics/`
