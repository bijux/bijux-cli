# Current Python Compatibility Notes

## Purpose
Describe the remaining Python-facing compatibility checks after the tracked
capture files were retired.

## Current checks

- Current Rust runtime vs current Python package:
  - `crates/bijux-cli-python/tests/python/test_runtime_parity.py`
  - `crates/bijux-cli/tests/integration/cli/plugins/plugin_command_parity.rs`
- Current Python package vs PyPI stable release `0.2.0`:
  - `crates/bijux-cli-python/tests/python/test_stable_release_compatibility.py`
  - run intentionally with `BIJUX_ENABLE_STABLE_PYPI_PARITY=1`

## Removed workflow

Tracked files under `artifacts/python-behavior/` and
`artifacts/current-python-behavior-lock.json` are no longer part of the repo.
`artifacts/` is disposable local output only.
