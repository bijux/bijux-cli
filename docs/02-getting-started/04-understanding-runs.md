# Understanding Runs

## Purpose
Define the run model a beginner needs for confident execution analysis.

## Context
Run interpretation is required for troubleshooting, replay, and diff.

## Explanation
Run essentials:
- each run has a unique run ID
- each run has lifecycle status transitions
- each run stores execution evidence

Beginner mental model:
- graph answers "what should happen"
- run answers "what happened this time"

Quick run review checklist:
1. capture run ID
2. check terminal status
3. inspect failed nodes if any
4. compare with prior run when behavior changed

## Examples
```bash
bijux-dag inspect run --run-id RUN_20260309_001
bijux-dag diff run --left RUN_20260309_001 --right RUN_20260309_002
```

## Guarantees
- Run identity and status interpretation are defined consistently with getting-started flow.
- The checklist is sufficient for first-pass diagnostics.

## Limitations
- Backend-specific storage details are not covered here.
- This is an operational guide, not schema contract text.

## Related
- `docs/02-getting-started/03-running-a-pipeline.md`
- `docs/02-getting-started/05-basic-troubleshooting.md`
- `docs/03-user-guide/04-run-history.md`
- `docs/06-specification/02-run-model.md`
