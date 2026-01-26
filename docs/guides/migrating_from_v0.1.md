# Migrating from v0.1

This guide highlights breaking changes and how to adapt.

## Summary

- Output routing is now explicit and stable
- CLI intent is fully resolved before execution
- Plugins require validated metadata

## Changes to watch

- Commands that relied on implicit defaults must now set flags explicitly
- Plugins must declare CLI compatibility
- Exit codes are stable and enforced

## Migration steps

1. Verify `bijux --help` and `bijux version` output
2. Update any scripts to pass explicit `--format`
3. Update plugins to include required metadata
