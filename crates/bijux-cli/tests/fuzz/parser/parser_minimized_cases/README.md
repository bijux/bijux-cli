# Minimized Parser Cases

This directory contains retained `.argv` files replayed by
`tests/routing/parser/parser_fuzz_regressions.rs`.

- Each non-comment line is one space-delimited argv sequence.
- Keep cases minimized to the smallest input that still reproduces the parser behavior under test.
- Use this directory for retained regressions, not for broad corpus exploration.
