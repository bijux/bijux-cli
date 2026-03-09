# Help Rendering Rules

This document freezes the first Rust help baseline rules.

## Rendering rules

1. Help output is always plain text, independent of `--format` machine-output flags.
2. Root help and subcommand help must be deterministic for identical inputs.
3. `--color never` must produce ANSI-free output.
4. Width-constrained rendering must remain valid and readable (`COLUMNS` env support).
5. Unknown command handling must return a non-zero exit and a stable error diagnostic.

## Ordering and naming

1. Root help command list follows the routed Rust command registry order.
2. Grouped command help (`cli`, `dev`) follows subcommand declaration order.
3. Alias forms that normalize to canonical routes must expose equivalent help text.

## Performance baseline

1. Root help should render under 1500ms in CI/test environments.
2. Performance regressions above this budget require explicit review.

## Compatibility notes

1. Python-vs-Rust help differences are allowed only when documented in parity reports.
2. Unreviewed command-tree drift in help output is treated as a regression.
