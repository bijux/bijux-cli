# Inspect Commands

## Purpose
Document inspect command usage for run and artifact diagnostics.

## Context
Inspect commands are the first diagnostic surface after execution anomalies.

## Explanation
Inspect command intents:
- run-level diagnostics
- artifact-level diagnostics
- operator-readable and machine-readable inspection outputs

Error handling conventions:
- not found: missing `run_id` or `artifact_id`
- invalid argument: malformed flags/values
- execution error: command runs but inspection retrieval fails

Exit code conventions (reference level):
- `0` success
- non-zero failure

## Examples
```bash
bijux-dag inspect run --run-id RUN_20260309_220
bijux-dag inspect artifact --artifact-id ART_20260309_902
```

```json
{
  "run_id": "RUN_20260309_220",
  "status": "failed",
  "failed_node_count": 1
}
```

## Guarantees
- Inspect usage paths for run and artifact are documented.
- Exit code and failure interpretation are explicitly addressed.

## Limitations
- Exact numeric non-zero code mapping can evolve by implementation.
- Deep backend internals are outside this reference page.

## Related
- `docs/04-cli-reference/03-run-commands.md`
- `docs/04-cli-reference/04-artifact-commands.md`
- `docs/03-user-guide/07-inspect-and-debug.md`
- `docs/04-cli-reference/06-diff-commands.md`
