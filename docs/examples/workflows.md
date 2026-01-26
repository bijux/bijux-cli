# Workflows

## Purpose
This document shows a stateful workflow and explains why each step matters.

## Scope
It covers core config and status flows only.

## What problem this solves
Examples without consequences teach syntax, not behavior.
This workflow teaches how output format and state changes interact.

## Why you should care
If you script bijux-cli, you must know which output is safe for machines.

## What confusion this removes
It removes the guesswork about when output is styled versus structured.

## Guarantees
Bijux guarantees:
1. `--format json` produces machine-readable output.
2. Config changes are reversible.

## How to Think About This
Treat each command as a state transition with a visible effect.

## Common Misunderstandings
- "Status output is always human-readable." It is not.

## Execution
Setup:

```bash
bijux config set mode=strict
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

Running with `--format json` disables styling and produces stable machine output.
This guarantees your parser sees only JSON, not terminal decorations.

Cleanup:

```bash
bijux config unset mode
```

## Failure Modes
- Invalid config keys exit with code 2.

## Design Rationale
We deliberately chose a stateful example because it reveals consequences.
Why not a one-line example? It hides the mutation and cleanup steps.

## Non-Goals
- Plugin workflows.
