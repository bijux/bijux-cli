# Inspect Commands

Document inspect command usage for run and artifact diagnostics.

Inspect commands are the first diagnostic step in troubleshooting workflows.

## Explanation
Common inspect operations:
- inspect run diagnostics.
- inspect artifact diagnostics.
- gather first-response evidence for debugging decisions.

Common flags:
- `--run-id <id>` run selector
- `--artifact-id <id>` artifact selector
- `--output <format>` where supported

Command lifecycle role:
- inspect is the first diagnostic command family after run execution.
- inspect output should drive whether replay or diff is needed next.

Command discovery:
- `bijux-dag inspect --help`
- `bijux-dag inspect run --help`
- `bijux-dag inspect artifact --help`

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
  "failed_node_count": 1,
  "first_failed_node": "transform"
}
```

```text
Inspect-driven debug flow:
1) inspect run --run-id ...
2) inspect artifact --artifact-id ...
3) replay --run-id ...
4) diff run --left ... --right ...
```

## Guarantees
- Inspect usage for run and artifact surfaces is documented and consistent.
- Failure interpretation and exit-code conventions are explicit.
- Command flow is aligned with user-guide debugging sequence.

## Limitations
- Exact non-zero code catalog is implementation-defined.
- Deep backend internals are not covered.

## Related
- `docs/04-cli-reference/03-run-commands.md`
- `docs/04-cli-reference/04-artifact-commands.md`
- `docs/04-cli-reference/06-diff-commands.md`
- `docs/03-user-guide/07-inspect-and-debug.md`

## Inspect targets and operator usage

Choose inspect target by diagnostic question:

- `inspect run`: use when you need execution status, failure scope, and node-level outcome summary.
- `inspect artifact`: use when you need identity, lineage, and evidence about produced outputs.

Operator usage pattern:

1. inspect run to localize failure or drift scope.
2. inspect artifact for outputs on the affected path.
3. escalate to replay/diff only after inspect evidence narrows hypotheses.

## Output modes and failure semantics

Output modes should be selected by consumer:

- human-readable mode for interactive triage.
- JSON mode for automation and incident artifact capture.

Failure semantics:

- unknown selectors (`run_id`, `artifact_id`) are lookup failures.
- malformed selector values are input failures.
- evidence retrieval errors indicate storage/runtime access issues and should be treated separately from execution failure.
