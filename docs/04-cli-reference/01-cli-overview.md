# Cli Overview

## Purpose
Provide the command hierarchy, output conventions, and reference navigation model for bijux-dag CLI usage.

## Context
This page is the single entrypoint for command-family docs in this section.

## Explanation
CLI hierarchy:
- `dag`: graph definition operations
- `run`: execution and run history operations
- `artifact`: artifact listing and inspection operations
- `inspect`: diagnostic introspection operations
- `diff`: graph/run/artifact comparison operations
- `replay`: validation and drift-detection operations
- `bundle`: export/import portability operations

Shared reference conventions:
- IDs are explicit (`run_id`, `artifact_id`).
- Automation should prefer machine-readable output flags when available.
- Help discovery follows command depth:
  - `bijux-dag --help`
  - `bijux-dag <group> --help`
  - `bijux-dag <group> <command> --help`

Shared error and exit-code model:
- `0`: successful command completion.
- non-zero: validation/input/runtime failure.

## Examples
```bash
bijux-dag --help
bijux-dag run --help
bijux-dag run history --help
```

## Guarantees
- CLI hierarchy is documented in one place.
- Shared conventions are reused across all command-family docs.

## Limitations
- Subcommand details are owned by each command-family document.
- Exact non-zero exit-code mapping can vary by implementation version.

## Related
- `docs/04-cli-reference/02-dag-commands.md`
- `docs/04-cli-reference/03-run-commands.md`
- `docs/04-cli-reference/05-inspect-commands.md`
- `docs/04-cli-reference/06-diff-commands.md`
