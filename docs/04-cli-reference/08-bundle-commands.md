# Bundle Commands

## Purpose
Document bundle export/import commands used for portability workflows.

## Context
Bundle commands support moving workflow context between environments.

## Explanation
Bundle command intents:
- `bundle export` to package run/graph context
- `bundle import` to load packaged context in target environment

Options and flags pattern:
- export uses source selectors (`--run-id`, optional scope flags)
- import uses bundle path (`--bundle <path>`)

Error handling guidance:
- unreadable bundle path: input failure
- invalid bundle format: validation failure
- unsupported portability scenario: compatibility failure

## Examples
```bash
bijux-dag bundle export --run-id RUN_20260309_220 --out ./exports/run220.bundle
bijux-dag bundle import --bundle ./exports/run220.bundle
```

```json
{
  "bundle_path": "./exports/run220.bundle",
  "import_status": "ok"
}
```

## Guarantees
- Export/import flow is documented with concrete commands.
- Compatibility and failure interpretation is explicit.

## Limitations
- This page does not define binary bundle schema internals.
- Portability remains constrained by backend support policy.

## Related
- `docs/04-cli-reference/07-replay-commands.md`
- `docs/03-user-guide/08-bundles-and-portability.md`
- `docs/07-operations/05-backend-support.md`
- `docs/06-specification/03-artifact-model.md`
