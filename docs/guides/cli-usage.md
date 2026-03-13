# CLI Usage

## Purpose
This guide explains how to use the bijux-cli command line effectively. It exists to help you choose the right flags and output formats for real-world workflows.

## Scope
It covers core command structure, common flags, and output formats. It does not describe every command in detail; use the reference for exhaustive command lists.

## Command Structure
Bijux commands follow a consistent pattern: `bijux <command> [subcommand] [args] [flags]`. This consistency is designed to make the CLI predictable across different command groups.

## Output Formats
Use `--format json` when you need machine-readable output and `--format yaml` when you prefer human-readable structured output. Structured formats disable styling to guarantee consistent parsing.

## Color and Quiet Modes
Color is a presentation concern and does not affect results. Quiet mode reduces non-essential output but does not change exit codes or structured output semantics.

## Examples
The following example requests a JSON response to ensure parsing stability in scripts:

```bash
bijux status --format json
```

## Why This Matters
Choosing explicit output formats and flags prevents surprises in CI and automation. The CLI is designed to make these choices explicit so behavior remains stable over time.

## References
- [Commands](../reference/commands.md)
- [Concepts overview](../concepts/index.md)
