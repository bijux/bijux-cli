# Quickstart

## Purpose
This document guarantees a complete config cycle.

## Scope
It covers a minimal CLI workflow only.

## Core Concepts
- Set, get, list, and unset a config value.

## Invariants
- Structured output respects `--format`.
- Exit codes are stable.

## Execution
```bash
bijux config set foo=bar
bijux config get foo
bijux config list --format json
bijux config unset foo
```

## Failure Modes
- Invalid key syntax exits with code 2.

## Design Rationale
- Alternatives: status-only example.
- Rejected because it does not mutate state.

## Non-Goals
- Plugin usage.
