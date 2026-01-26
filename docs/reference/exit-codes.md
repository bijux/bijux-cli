# Exit codes

## Purpose
This document guarantees the CLI exit codes.

## Scope
It lists codes and meanings only.

## Core Concepts
- Exit codes are stable.

## Invariants
- Exit codes never change with output format.

## Execution
| Code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | Internal error |
| 2 | Usage or user input error |
| 3 | ASCII or encoding error |
| 130 | Aborted by user |

## Failure Modes
- New codes require explicit documentation.

## Design Rationale
- Alternatives: per-command codes.
- Rejected because they reduce predictability.

## Non-Goals
- OS-specific signal mappings.
