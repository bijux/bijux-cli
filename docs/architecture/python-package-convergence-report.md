# Python Package Convergence Report

Scope: stable parity and behavior coverage.

## Summary

The Python distribution now executes through the same Rust route graph and runtime semantics used by the Rust CLI binary.

## Completed items

- `361`: complete. Python package command execution defaults to Rust-backed runtime (`_native` when available, subprocess runtime fallback otherwise).
- `362`: complete. Wrapper layers retain thin facade behavior only and avoid duplicate command-law logic.
- `363`: complete. Bridge execution path uses core `run_app` through `crates/bijux-cli-python/src/bindings.rs`.
- `364`: complete. End-to-end invocation parity covered via runtime subprocess comparisons in `crates/bijux-cli-python/tests/python/test_runtime_parity.py`.
- `365`: complete. `python -m bijux_cli_py` parity checks are covered in runtime tests.
- `366`: complete. `pipx` install path behavior is covered by install guidance and path ambiguity diagnostics; direct runtime parity surfaces are shared with console-script invocation.
- `367`: complete. Version parity checks are covered between runtime binary and Python facade.
- `368`: complete. Help parity checks are covered between runtime binary and Python facade.
- `369`: complete. Invalid-command exit and stream behavior parity checks are covered.
- `370`: complete. Plugin list behavior parity smoke checks are covered.
- `371`: complete. REPL startup parity smoke checks are covered.
- `372`: complete. Public import facade surface remains explicit and tested.
- `373`: complete. Extension-load and missing-runtime failure behavior is tested.
- `374`: complete. Migration warnings and compatibility messaging are implemented and tested.
- `375`: complete. Script entrypoint contract test added (`bijux -> bijux_cli_py.cli:main`).
- `376`: complete. Script naming contract is covered in packaging tests.
- `377`: complete. Wheel metadata consistency checks added for project name/version/python range and module name.
- `378`: complete. This report is the Python-package convergence artifact.
- `379`: complete. Baseline freeze is recorded in `docs/architecture/python-package-baseline.md`.
- `380`: complete. Python API retention/deprecation decisions are recorded in `docs/architecture/python-public-api-lifecycle.md`.

## Evidence

- `crates/bijux-cli-python/tests/python/test_runtime_parity.py`
- `crates/bijux-cli-python/tests/python/test_runtime_resilience.py`
- `crates/bijux-cli-python/tests/python/test_packaging_contracts.py`
- `crates/bijux-cli-python/src/bindings.rs`

## Remaining follow-ups after baseline

- Add packaged wheel install-and-run checks in CI matrix for Linux/macOS/Windows using built wheel artifacts.
- Expand plugin write-path and full REPL interactive parity checks through Python package command surfaces.
