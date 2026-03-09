# Cli Overview

## Purpose
Provide the command hierarchy, output conventions, and reference navigation model for bijux-dag CLI usage.

## Context
This page is the single entrypoint for command-family docs in this section.

## Explanation
Command hierarchy:
- `dag`: definition validation and graph inspection.
- `run`: execution and run-history surfaces.
- `artifact`: artifact listing and artifact inspection.
- `inspect`: run and artifact diagnostic views.
- `diff`: graph, run, and artifact comparison scopes.
- `replay`: deterministic validation against baseline run context.
- `bundle`: portability export/import workflows.

Shared reference conventions:
- IDs are explicit (`run_id`, `artifact_id`).
- Automation should prefer machine-readable output flags when available.
- Help discovery follows command depth:
  - `bijux-dag --help`
  - `bijux-dag <group> --help`
  - `bijux-dag <group> <command> --help`

CLI grammar overview:
- root grammar: `bijux-dag <group> <action> [flags]`.
- selector pattern: `--run-id <id>` or `--artifact-id <id>` when targeting existing entities.
- comparator pattern: `--left <value> --right <value>` for diff commands.
- output pattern: `--output json` for machine-readable pipelines where available.

Command lifecycle model:
1. define and validate DAG (`dag`).
2. execute and record run (`run`).
3. inspect evidence (`inspect`, `artifact`).
4. validate and compare (`replay`, `diff`).
5. transfer context when needed (`bundle`).

Shared error and exit model:
- `0`: successful completion.
- non-zero validation category: malformed input, invalid graph, missing required flags.
- non-zero lookup category: unknown run/artifact IDs or missing referenced files.
- non-zero runtime category: execution/replay/import failures.

## Examples
```bash
# Top-level discovery
bijux-dag --help

# Group-level discovery
bijux-dag run --help
bijux-dag run history --help

# Common machine-readable invocation pattern
bijux-dag run history --limit 20 --output json
```

```text
Reference navigation:
1) read this page for hierarchy and shared conventions
2) open command-family page
3) use --help at matching depth for exact runtime syntax
```

## Guarantees
- CLI hierarchy is documented in one place.
- Shared conventions are reused across all command-family docs.
- Grammar and lifecycle mental model are explicit for operator workflows.

## Limitations
- Subcommand details are owned by each command-family document.
- Exact non-zero exit-code mapping can vary by implementation version.

## Related
- `docs/04-cli-reference/02-dag-commands.md`
- `docs/04-cli-reference/03-run-commands.md`
- `docs/04-cli-reference/05-inspect-commands.md`
- `docs/04-cli-reference/06-diff-commands.md`
