# Python Package Baseline

This document freezes the first Python-package convergence baseline.

## Baseline guarantees

- Python package command execution maps to the Rust runtime command graph.
- The `bijux` console script remains the canonical Python-installed entrypoint.
- `python -m bijux_cli_py` behavior is parity-checked against runtime binary behavior for baseline commands.
- Version/help/error stream behavior for covered commands is tested against runtime behavior.
- Plugin list and REPL startup smoke parity are covered through the package facade.
- Packaging contracts for script mapping and wheel module naming are tested.

## Baseline constraints

- Interactive REPL UI parity (prompt toolkit details) is outside this baseline.
- Full cross-platform wheel integration tests are tracked as CI follow-up work.
- Python-only legacy wrapper assumptions are supported through migration warnings, not through duplicate command execution logic.

## Change policy

Any change that alters package invocation semantics, script ownership, or runtime parity for covered commands requires:

1. Updated tests in `packages/bijux-cli-py/tests`.
2. Updated convergence report (`python-package-convergence-report.md`).
3. Explicit compatibility decision in release documentation.
