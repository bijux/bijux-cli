# Config Root Listing Parity Report

Scope: tasks 101-120.

## Decision

Root `config` in Rust lists active file-backed config entries only.

- It does not include path metadata fields.
- Path metadata remains exposed through `cli paths` and diagnostics commands.

This matches Python root `config` baseline behavior where output is an entry map.

## Coverage

- Core behavior tests:
  - `crates/bijux-cli/tests/config_root_listing.rs`
- Binary behavior tests:
  - `crates/bijux-cli/tests/cli_surface/config/config_root_parity.rs`
- Snapshot artifacts:
  - `crates/bijux-cli/tests/cli_surface/snapshots/config_root_text.txt`
  - `crates/bijux-cli/tests/cli_surface/snapshots/config_root_json_pretty.txt`
  - `crates/bijux-cli/tests/cli_surface/snapshots/config_root_json_compact.txt`
  - `crates/bijux-cli/tests/cli_surface/snapshots/config_root_yaml_pretty.txt`

## Task matrix

- `101`: complete (root `config` listing implemented in `core::config::service`).
- `102`: complete (lists active stored entries).
- `103`: complete (file-backed values selected; documented here).
- `104`: complete (text snapshot).
- `105`: complete (JSON snapshot).
- `106`: complete (YAML snapshot).
- `107`: complete (pretty JSON snapshot).
- `108`: complete (compact JSON snapshot).
- `109`: complete (pretty YAML snapshot).
- `110`: complete (quiet-mode test).
- `111`: complete (no-color test).
- `112`: complete (stdout/stderr routing tests).
- `113`: complete (Python-vs-Rust parity test).
- `114`: complete (exit-code parity included in parity tests).
- `115`: complete (empty-file test).
- `116`: complete (malformed-file test).
- `117`: complete (duplicate-key test).
- `118`: complete (overridden-path test).
- `119`: complete (trace-mode stability test).
- `120`: complete (root `config` marked parity-complete in this report).
