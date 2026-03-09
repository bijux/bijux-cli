# Artifact Commands

Document artifact command usage for listing and inspecting output objects.

Artifact commands are used after run completion to inspect output integrity and lineage context.

## Explanation
Common artifact operations:
- list artifacts for a run.
- inspect a specific artifact.
- retrieve artifact lineage details for diagnostics.

Common flags:
- `--run-id <id>` for artifact listing scope
- `--artifact-id <id>` for direct inspection
- `--output <format>` where supported

Command lifecycle role:
- artifact commands are post-run evidence tools.
- use list first to find candidate artifacts, then inspect for lineage and identity details.

Command discovery:
- `bijux-dag artifact --help`
- `bijux-dag artifact list --help`
- `bijux-dag artifact inspect --help`

Error handling guidance:
- unknown artifact ID: lookup error
- malformed selection flags: input error
- missing run-scoped artifacts: lookup or empty-result condition depending on command behavior

## Examples
```bash
bijux-dag artifact list --run-id RUN_20260309_220 --output json
bijux-dag artifact inspect --artifact-id ART_20260309_902 --output json
```

```json
{
  "artifact_id": "ART_20260309_902",
  "run_id": "RUN_20260309_220",
  "node_id": "transform",
  "hash": "sha256:2f67..."
}
```

```text
Artifact discovery flow:
1) artifact list --run-id RUN_...
2) select artifact_id from list
3) artifact inspect --artifact-id ART_...
```

## Guarantees
- Artifact command usage is identity-first and lineage-aware.
- Examples are aligned with artifact user-guide workflows.
- Command flow reflects practical list-then-inspect usage.

## Limitations
- Hashing implementation internals are not defined in this page.
- Storage backend mechanics are documented in architecture/spec docs.

## Related
- `docs/04-cli-reference/01-cli-overview.md`
- `docs/04-cli-reference/03-run-commands.md`
- `docs/04-cli-reference/06-diff-commands.md`
- `docs/03-user-guide/03-artifacts.md`
- `docs/06-specification/06-artifact-identity.md`
