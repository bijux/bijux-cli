# Config Get Parity Report

Scope: stable parity and behavior coverage.

## Decisions

- `config get` remains a direct value read command with Python-matching not-found semantics.
- Current baseline returns `key`, `value`, and `source_path`.
- Broader source metadata exposure is deferred until after parity lock.

## Coverage

- Core behavior tests:
  - `crates/bijux-cli/tests/integration/cli/config/config_parity.rs`
  - `crates/bijux-cli/tests/integration/cli/config/config_key_value_parity.rs`
  - `crates/bijux-cli/tests/integration/cli/config/config_get_performance.rs`
- Binary behavior tests:
  - `crates/bijux-cli/tests/integration/cli/config/config_get_parity.rs`
  - `crates/bijux-cli/tests/integration/cli/config/config_python_compatibility.rs`
- Snapshot artifacts:
  - `crates/bijux-cli/tests/data/golden/cli_surface/config_get_text.txt`
  - `crates/bijux-cli/tests/data/golden/cli_surface/config_get_json_pretty.txt`
  - `crates/bijux-cli/tests/data/golden/cli_surface/config_get_json_compact.txt`
  - `crates/bijux-cli/tests/data/golden/cli_surface/config_get_yaml_pretty.txt`

## Coverage matrix

- `121`: complete (`cli config get` implemented in Rust core path).
- `122`: complete (missing key returns usage failure semantics matching Python baseline).
- `123`: complete (source metadata expansion deferred; current baseline retained).
- `124`: complete (text snapshot).
- `125`: complete (JSON snapshot).
- `126`: complete (YAML snapshot).
- `127`: complete (pretty/compact JSON assertions).
- `128`: complete (found-key test).
- `129`: complete (missing-key test).
- `130`: complete (invalid-key syntax test).
- `131`: complete (key normalization test).
- `132`: complete (path override test).
- `133`: complete (malformed config file behavior test).
- `134`: complete (quiet-mode test).
- `135`: complete (no-color mode test).
- `136`: complete (Python-vs-Rust parity tests for success and missing key).
- `137`: complete (exit-code parity checks in parity tests).
- `138`: complete (stderr/stdout routing parity checks).
- `139`: complete (performance sanity benchmark guard).
- `140`: complete (post-parity follow-up list published separately).
