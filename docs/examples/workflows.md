# Workflows

## Purpose
This document guarantees a complete CLI workflow with consequences.

## Scope
It covers core config and status flows only.

## Core Concepts
- Structured output is explicit.
- Config mutations are reversible.

## Invariants
- `--format json` produces machine-readable output.
- Exit codes are stable.

## Execution
Setup:

```bash
bijux config set mode strict
```

Command:

```bash
bijux status --format json
```

Output:

```json
{"version":"...","status":"ok"}
```

Implication:

- JSON output disables styling and is safe for scripts.

Cleanup:

```bash
bijux config unset mode
```

## Failure Modes
- Invalid config keys exit with code 2.

## Design Rationale
- Alternatives: status-only examples.
- Rejected because they do not show state changes.

## Non-Goals
- Plugin workflows.
