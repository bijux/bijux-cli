# Concepts

## Purpose
This section guarantees the core decisions and invariants of bijux-cli.

## Scope
It does not provide step-by-step guides or reference tables.

## What problem this solves
Without a single source of truth, decisions drift and tests fail.

## Why you should care
If you change behavior, these guarantees tell you what must remain stable.

## What confusion this removes
It removes ambiguity about where decisions are made.

## Guarantees
Bijux guarantees:
1. Concept docs are authoritative for CLI behavior.

## How to Think About This
Treat these docs as the contract for user-visible behavior.

## Common Misunderstandings
- "Guides define behavior." They do not. Concepts do.

## Execution
- Architecture: architecture.md
- Execution model: execution-model.md
- Precedence: precedence.md
- Exit policy: exit-policy.md
- Logging: logging.md
- Plugin lifecycle: plugin-lifecycle.md

## Failure Modes
- Conflicts between concepts and code are defects.

## Design Rationale
We deliberately chose a concept set so behavior stays centralized.
Why not embed this in guides? It leads to duplication and drift.

## Non-Goals
- Tutorials or quickstart material.
