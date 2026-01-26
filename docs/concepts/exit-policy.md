# Exit Policy

## Purpose
This document defines the exit policy for bijux-cli. It describes how exit codes are assigned and what guarantees users can rely on when integrating bijux-cli into automation.

## Scope
It covers exit code mapping and error classification. It does not cover command output formatting or logging details beyond what is needed to explain exit behavior.

## Core Concepts
Exit policy is a contract: each error class maps to a stable exit code, and successful commands always return zero. The policy applies uniformly across CLI commands.

## How to Think About This
Treat exit codes as part of the public API. If a command fails, the exit code is the authoritative signal for automation, and output is supplemental. The policy exists to ensure scripts remain stable even as internal implementations change.

## Invariants
- Success returns exit code 0.
- Known error classes map to fixed exit codes.
- Formatting flags do not alter exit codes.

## Failure Modes
If an unexpected exception occurs, the CLI emits a structured error and returns a general failure exit code. This preserves a deterministic signal even when the underlying error is not classified.

## Design Rationale
Stable exit codes are essential for CI and automation. By fixing the mapping, we reduce the risk of silent behavioral changes and make failures debuggable.

## Non-Goals
This document does not define error payload formats. It focuses only on exit behavior.

## References
- Implementation: `src/bijux_cli/core/exit_policy.py`
- Regression coverage: `tests/regression/test_exit_policy.py`
