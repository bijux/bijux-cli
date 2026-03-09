# Running A Pipeline

## Purpose
Explain the default run workflow from execution to replay/diff diagnostics.

## Context
This document bridges beginner DAG authoring and operational debugging.

## Explanation
Standard flow:
1. `run`
2. `inspect run`
3. `replay`
4. `diff run`

Command cheat sheet:
- run: `bijux-dag run --dag <path>`
- inspect: `bijux-dag inspect run --run-id <id>`
- replay: `bijux-dag replay --run-id <id>`
- diff: `bijux-dag diff run --left <id> --right <id>`

Run lifecycle (mental model):
- created -> executing -> completed/failed

Readability and flow checks applied:
- command order mirrors real operational sequence
- one command objective per step
- no mixed conceptual and contract language

## Examples
```bash
bijux-dag run --dag ./examples/first.dag.json
bijux-dag inspect run --run-id RUN_20260309_001
bijux-dag replay --run-id RUN_20260309_001
bijux-dag diff run --left RUN_20260309_001 --right RUN_20260309_002
```

## Guarantees
- This guide documents a coherent first-response run workflow.
- Replay and diff usage are integrated into normal operations, not separate tracks.

## Limitations
- Advanced release/ci lane behavior is not covered.
- Field-level diff classes are specified elsewhere.

## Related
- `docs/02-getting-started/04-understanding-runs.md`
- `docs/02-getting-started/05-basic-troubleshooting.md`
- `docs/03-user-guide/05-replay.md`
- `docs/03-user-guide/06-diff.md`
