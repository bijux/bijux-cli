# REPL Parity

## Purpose
Define parity requirements between non-interactive CLI commands and REPL command execution.

## Scope
This document governs behavioral parity, not UI styling.

## Core Concepts
- REPL is an alternate interaction surface over the same command model.

## Invariants
- Commands available in non-interactive mode remain available in REPL unless explicitly documented as non-interactive-only.
- Exit-code semantics map to REPL status reporting semantics consistently.
- Config, env, and flag precedence semantics are shared between CLI and REPL evaluation.
- Structured output requests in REPL use the same envelope contracts.
- REPL meta-commands must use the reserved prefix `:` and must never conflict with normal command namespaces.

## Failure Modes
- Divergent REPL command semantics are treated as compatibility defects.

## Design Rationale
- Users must be able to move between REPL and scripts without relearning behavior.

## Non-Goals
- Freezing visual prompt style or line-editing UX.
