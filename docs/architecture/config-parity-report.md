# Config Parity Report

Scope: current Rust coverage for `bijux cli config` command behavior, parity
tests, and remaining known gaps.

## Covered Commands

| Command | Status | Notes |
|---|---|---|
| `config` | parity-complete | file-backed listing baseline |
| `config get` | parity-complete | not-found, stream routing, and format coverage present |
| `config set` | parity-complete | `KEY=VALUE`, stdin fallback, validation, and write safety covered |
| `config unset` | parity-complete | existing, missing, and malformed coverage present |
| `config clear` | parity-complete | empty, missing, and write-failure coverage present |
| `config reload` | parity-complete | success, malformed, and missing coverage present |
| `config export` | parity-complete | path-required and text rejection aligned |
| `config load` | parity-complete | valid, malformed, duplicate, and path handling covered |

## Current Behavior Summary

- `config get` normalizes keys, honors environment overrides before file values,
  supports `--config-path`, and returns deterministic not-found failures.
- `config set` accepts `KEY=VALUE`, applies Python-aligned validation, creates
  missing parents, writes atomically, and preserves unrelated settings.
- `config unset`, `clear`, and `reload` follow the current mutation and
  validation rules documented by parity tests and snapshots.
- `config export` and `load` keep the dotenv-based baseline contract rather than
  inventing a different serialized format.
- Success payloads stay on stdout and failure payloads stay on stderr.

## Coverage Sources

- `crates/bijux-cli/tests/integration/cli/config/config_root_listing.rs`
- `crates/bijux-cli/tests/integration/cli/config/config_parity.rs`
- `crates/bijux-cli/tests/integration/cli/config/config_get_parity.rs`
- `crates/bijux-cli/tests/integration/cli/config/config_set_parity.rs`
- `crates/bijux-cli/tests/integration/cli/config/config_mutation_parity.rs`
- `crates/bijux-cli/tests/integration/cli/config/config_export_load_parity.rs`
- `crates/bijux-cli/tests/integration/cli/config/config_python_compatibility.rs`

Snapshot coverage lives under
`crates/bijux-cli/tests/data/golden/cli_surface/`.

## Remaining Gaps

- Cross-process lock contention parity is still documented rather than modeled
  in the write path.
- The command surface is parity-locked, but future UX changes still require
  parity-impact review and matching snapshot updates.

## Deferred Follow-Up

These items are intentionally not part of the parity baseline:

- optional source-trace metadata for reads
- suggestion hints for missing keys
- typed coercion helpers for common value shapes
- cache-aware read paths for long-lived sessions
- richer write diagnostics once contention handling changes

## Generated Evidence

Use generated artifacts for current status instead of extending this page with
manual status tables:

- `artifacts/parity/config_parity_report.json`
- `artifacts/parity/command_parity_matrix.json`
- `artifacts/status/status_state_behavior_coverage.json`

Use `bijux dev cli status --format json` for the current machine-readable view.
