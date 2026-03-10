# Config Export Load Parity Report

Scope: tasks `181-193` for Rust config parity.

## Behavior Decisions

- `config export PATH`
  - Copies active config state into `PATH` using dotenv layout.
  - Returns metadata payload with `status`, `file`, and `format`.
  - Keeps Python baseline behavior where `--format text` is rejected.
- `config load PATH`
  - Loads external dotenv file from `PATH` into active config path.
  - Replaces active config with loaded snapshot.
  - Wraps load failures as `Failed to load config: ...` and maps to usage-style exit (`2`).

## Coverage

- Binary parity suite:
  - `crates/bijux-cli-core/tests/bin_surface/config_export_load_parity.rs`
- Snapshots:
  - `crates/bijux-cli-core/tests/snapshots/config_export_json_compact.txt`
  - `crates/bijux-cli-core/tests/snapshots/config_export_yaml_pretty.txt`
  - `crates/bijux-cli-core/tests/snapshots/config_export_text_error.txt`

## Completed Tasks

- `181`: complete (`config export` implementation).
- `182`: complete (export semantics fixed to dotenv-file target; response format metadata is `auto`).
- `183`: complete (matched Python baseline behavior for path requirement and text-format rejection).
- `184`: complete (text-mode export error snapshot).
- `185`: complete (JSON snapshot).
- `186`: complete (YAML snapshot).
- `187`: complete (Python-vs-Rust parity tests for export exit/streams).
- `188`: complete (`config load` implementation).
- `189`: complete (valid external file load tests).
- `190`: complete (malformed external file tests).
- `191`: complete (duplicate-key external file tests; last value wins).
- `192`: complete (path traversal-style path load and unreadable-file handling tests).
- `193`: complete (Python-vs-Rust parity tests for load exit/streams).

## Notes

Current baseline keeps Python-compatible behavior where loading a missing file resolves as an empty snapshot load rather than a hard failure.
