# Stdout/Stderr Rules

## Purpose
Define deterministic stream usage for success, errors, and diagnostics.

## Scope
This document governs routing of command output across streams.

## Core Concepts
- Stream routing is a compatibility contract for shell pipelines.

## Invariants
- Success payloads are written to `stdout`.
- Error payloads and fatal diagnostics are written to `stderr`.
- Non-fatal debug logging is written to `stderr`.
- `--quiet` suppresses non-essential informational text but must not suppress required machine-readable output.
- Commands must not split one structured payload across both streams.

## Failure Modes
- Emitting success on `stderr` or errors on `stdout` is a contract violation.

## Design Rationale
- Predictable stream semantics make shell scripting reliable.

## Non-Goals
- Defining shell redirection strategies for end users.
