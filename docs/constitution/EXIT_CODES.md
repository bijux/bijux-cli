# Exit Codes

## Purpose
Define stable exit-code semantics for automation and interactive use.

## Scope
This document is normative for exit-code mapping.

## Core Concepts
- Exit codes are part of the public compatibility contract.

## Invariants
- `0`: success.
- `1`: internal failure.
- `2`: usage or input validation failure.
- `3`: encoding or serialization failure.
- `130`: interrupted by user signal.
- Output format and color settings do not change exit-code mapping.

## Failure Modes
- Any new exit code requires constitutional documentation and release-note notice.

## Design Rationale
- Deterministic exit behavior is mandatory for CI and scripts.

## Non-Goals
- OS-specific signal passthrough beyond documented mappings.
