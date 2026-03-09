# Inspect Commands

## Purpose
Document inspect command usage for run and artifact diagnostics.

## Context
Inspect commands are the first diagnostic step in troubleshooting workflows.

## Explanation
Common inspect operations:
- inspect run diagnostics
- inspect artifact diagnostics

Common flags:
- `--run-id <id>` run selector
- `--artifact-id <id>` artifact selector
- `--output <format>` where supported

Exit and error conventions:
- `0`: inspection completed
- non-zero: input, lookup, or runtime retrieval failure

## Examples
```bash
bijux-dag inspect run --run-id RUN_20260309_220 --output json
bijux-dag inspect artifact --artifact-id ART_20260309_902 --output json
```

```json
{
  "run_id": "RUN_20260309_220",
  "status": "failed",
  "failed_node_count": 1
}
```

## Guarantees
- Inspect usage for run and artifact surfaces is documented and consistent.
- Failure interpretation and exit-code conventions are explicit.

## Limitations
- Exact non-zero code catalog is implementation-defined.
- Deep backend internals are not covered.

## Related
- `docs/04-cli-reference/03-run-commands.md`
- `docs/04-cli-reference/04-artifact-commands.md`
- `docs/04-cli-reference/06-diff-commands.md`
- `docs/03-user-guide/07-inspect-and-debug.md`
