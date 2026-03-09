# Artifact Commands

## Purpose
Document artifact command usage for listing, inspecting, and comparing output objects.

## Context
Artifact commands are used after runs complete and outputs are available.

## Explanation
Artifact command intents:
- retrieve artifact identifiers and metadata
- inspect artifact identity and lineage surfaces
- support artifact diff workflows

Usage guidance:
- treat `artifact_id` as primary handle
- inspect lineage before declaring output anomalies

Common option pattern:
- `--artifact-id <id>` for direct lookup
- run-scoped options when selecting artifact sets

## Examples
```bash
bijux-dag artifact list --run-id RUN_20260309_220
bijux-dag artifact inspect --artifact-id ART_20260309_902
```

```json
{
  "artifact_id": "ART_20260309_902",
  "run_id": "RUN_20260309_220",
  "node_id": "transform"
}
```

## Guarantees
- Artifact command family is documented with identity-first handling.
- Lineage-aware inspection guidance is explicit.

## Limitations
- Hash algorithm internals are outside CLI reference scope.
- Storage layout details are covered in architecture/spec docs.

## Related
- `docs/04-cli-reference/01-cli-overview.md`
- `docs/03-user-guide/03-artifacts.md`
- `docs/04-cli-reference/06-diff-commands.md`
- `docs/06-specification/06-artifact-identity.md`
