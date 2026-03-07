# CLI backward compatibility policy

## Contract surface

Only the command tree and JSON envelope are compatibility contracts.
Human-readable plaintext output is intentionally non-contractual.

## Stable guarantees

- Top-level command names are stable once documented in `docs/CLI_COMMAND_TAXONOMY.md`.
- JSON envelope shape (`ok`, `command`, `data`, `diagnostics`) is stable.
- Documented non-zero exit code classes remain stable.

## Allowed changes

- Additive JSON fields under `data`.
- New subcommands that do not change existing command semantics.
- Improved plaintext wording and formatting.

## Breaking changes

- Removing or renaming documented command names.
- Changing JSON envelope shape.
- Reassigning established non-zero exit code classes.

## Governance

Any breaking CLI change requires a compatibility decision record and migration notes.
