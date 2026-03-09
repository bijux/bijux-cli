# Cli Overview

## Purpose
Provide the command hierarchy, usage model, and operational conventions for bijux-dag CLI usage.

## Context
This is the entrypoint for all CLI reference documents.

## Explanation
CLI hierarchy:
- `dag` for graph-level operations
- `run` for execution and history operations
- `artifact` for artifact surfaces
- `inspect` for introspection
- `diff` for comparison workflows
- `replay` for validation workflows
- `bundle` for portability workflows

Reference navigation:
- command-specific behavior is documented in each command-family page
- this page defines shared conventions only

Shared conventions:
- use explicit IDs (`run_id`, `artifact_id`) in follow-up commands
- prefer machine-readable output when automating
- inspect before replay/diff when failure context is unclear

Command quick reference:
```bash
bijux-dag dag --help
bijux-dag run --help
bijux-dag artifact --help
bijux-dag inspect --help
bijux-dag diff --help
bijux-dag replay --help
bijux-dag bundle --help
```

## Examples
```bash
# Hierarchical help discovery
bijux-dag --help
bijux-dag run --help
bijux-dag run history --help
```

## Guarantees
- CLI hierarchy and navigation are explicitly documented.
- Shared conventions are consistent across command-family docs.

## Limitations
- This page does not replace command-level semantics.
- Option-level details are defined in command-family documents.

## Related
- `docs/04-cli-reference/02-dag-commands.md`
- `docs/04-cli-reference/03-run-commands.md`
- `docs/04-cli-reference/04-artifact-commands.md`
- `docs/04-cli-reference/05-inspect-commands.md`
