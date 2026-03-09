# Bundle Commands

Use `bundle` commands to export/import execution context, then verify portability with inspect/replay/diff.

## Command roles

- `bundle export`: create transferable bundle from selected run context.
- `bundle import`: load transferable context into target environment.

## Core invocation patterns

```bash
bijux-dag bundle --help
bijux-dag bundle export --run-id RUN_20260309_220 --out ./exports/run220.bundle --output json
bijux-dag bundle import --bundle ./exports/run220.bundle --output json
```

If your build exposes verify-only or integrity-check subcommands, run those before mutating imports.

## Read-only versus mutating behavior

Mutating behavior:

- export writes bundle artifact,
- import writes local bundle-derived state.

Read-only behavior:

- help/discovery calls,
- verify-only checks (when supported) that do not persist imported state.

## Broken bundle example

```bash
bijux-dag bundle import --bundle ./exports/corrupted.bundle --output json
```

Expected interpretation:

- command fails validation or marks result as invalid,
- no portability claim should be made,
- rerun with known-good bundle or regenerate export from trusted run.

## Next reading

- Portability interpretation workflow: [Bundles And Portability](../03-user-guide/08-bundles-and-portability.md)
- Backend capability limits: [Backend Support](../07-operations/05-backend-support.md)
