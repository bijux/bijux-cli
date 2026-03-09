# Diff

## Purpose
Show how to compare graph, run, and artifact behavior using structured diff workflows.

## Context
Diff is the main tool for answering "what changed" after drift or regression.

## Explanation
Diff modes:
- graph diff: definition-level changes
- run diff: execution outcome and behavior changes
- artifact diff: output-level changes

Diff classification guidance:
- expected change: planned/configured difference
- suspicious change: unplanned and unexplained divergence
- breaking change: contract-relevant mismatch

Operational diff workflow:
1. choose comparable entities
2. run appropriate diff command
3. classify each reported difference
4. decide whether replay or remediation is required

## Examples
```bash
# Graph diff
bijux-dag diff graph --left ./pipelines/a.dag.json --right ./pipelines/b.dag.json

# Run diff
bijux-dag diff run --left RUN_20260309_001 --right RUN_20260309_002

# Artifact diff
bijux-dag diff artifact --left ART_001 --right ART_002
```

## Guarantees
- Diff usage is documented across graph, run, and artifact scopes.
- Classification guidance is explicit for operational triage.

## Limitations
- Exact field-by-field diff semantics are specified in contract docs.
- This guide does not replace deeper forensic workflows.

## Related
- `docs/03-user-guide/05-replay.md`
- `docs/03-user-guide/07-inspect-and-debug.md`
- `docs/06-specification/08-diff-semantics.md`
- `docs/06-specification/01-dag-model.md`
