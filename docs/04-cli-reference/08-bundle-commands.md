# Bundle Commands

Document bundle export/import command usage for portability workflows.

Bundle commands package and transfer workflow context across environments.

## Explanation
Bundle operations:
- `bundle export`
- `bundle import`

Common flags:
- export: source selectors (`--run-id`, output path)
- import: bundle selector (`--bundle <path>`)
- `--output <format>` where supported

Command lifecycle role:
- export after run completion and evidence verification.
- import in target environment before replay/diff portability checks.

Command discovery:
- `bijux-dag bundle --help`
- `bijux-dag bundle export --help`
- `bijux-dag bundle import --help`

Error handling guidance:
- unreadable path: input error
- invalid bundle: validation error
- unsupported environment mapping: compatibility error
- missing bundle file: filesystem lookup error

## Examples
```bash
bijux-dag bundle export --run-id RUN_20260309_220 --out ./exports/run220.bundle --output json
bijux-dag bundle import --bundle ./exports/run220.bundle --output json
```

```json
{
  "bundle_path": "./exports/run220.bundle",
  "import_status": "ok",
  "bundle_id": "BUNDLE_20260309_220"
}
```

```text
Bundle portability command flow:
1) bundle export --run-id ...
2) transfer bundle
3) bundle import --bundle ...
4) replay and diff for equivalence decision
```

## Guarantees
- Export/import flow is documented as concrete portability path.
- Failure interpretation categories are explicit.
- Output examples support scripted portability pipelines.

## Limitations
- Bundle schema internals are specified outside CLI reference.
- Portability guarantees depend on backend support boundaries.

## Related
- `docs/04-cli-reference/07-replay-commands.md`
- `docs/03-user-guide/08-bundles-and-portability.md`
- `docs/07-operations/05-backend-support.md`
- `docs/06-specification/03-artifact-model.md`

## Precise bundle command roles

- `bundle export`: creates a transferable evidence package from selected run context.
- `bundle import`: materializes transferable evidence into local context for verification workflows.

## Export, import, verify-only, and fsck-like workflows

```bash
bijux-dag bundle export --run-id RUN_20260309_220 --out ./exports/run220.bundle --output json
bijux-dag bundle import --bundle ./exports/run220.bundle --output json
```

Verification-oriented usage (implementation dependent):

- verify-only import mode validates bundle integrity without accepting mutable state changes.
- fsck-like integrity checks validate bundle structure and evidence consistency before replay.

## Mutating versus read-only behavior

Treat command effects explicitly:

- mutating: export creates/updates bundle artifacts; import writes local bundle-derived state.
- read-only: help/discovery commands and verification-only checks that do not persist changes.

When in doubt, run verification-first and capture JSON diagnostics before mutating import flows.
