# Config Unset Clear Reload Parity Report

Scope: tasks `161-180` for Rust configuration parity.

## Implementation Decisions

- `config unset`: real mutation path in Rust core that removes normalized key when present.
- `config clear`: real mutation path that removes active config file and reports removed key count.
- `config reload`: compatibility-safe reload operation that re-reads and validates active config file.
  There is no long-lived in-memory cache in this baseline; reload validates current disk state.

## Completed Coverage

- `161-164`: implemented `unset` with tests for existing key, missing key, and malformed key.
- `165`: output snapshot for `unset` in text mode.
- `166`: Python-vs-Rust parity checks for `unset` exit and stream behavior.
- `167-171`: implemented `clear` with tests for non-empty, already empty, missing file, and write failure.
- `172`: output snapshot for `clear` in text mode.
- `173`: Python-vs-Rust parity checks for `clear` exit and stream behavior.
- `174-178`: implemented `reload` with success, malformed, and missing-file tests.
- `179`: output snapshot for `reload` in text mode.
- `180`: parity checks for `reload` success-path exit and stream behavior.

## Test Artifacts

- Core integration coverage:
  - `crates/bijux-cli/tests/config_parity.rs`
- Binary parity coverage:
  - `crates/bijux-cli/tests/bin_surface/config_mutation_parity.rs`
- Snapshots:
  - `crates/bijux-cli/tests/snapshots/config_unset_text.txt`
  - `crates/bijux-cli/tests/snapshots/config_clear_text.txt`
  - `crates/bijux-cli/tests/snapshots/config_reload_text.txt`

## Remaining Config Scope

Config command baseline parity is now frozen. Remaining work is post-parity improvements and explicit UX changes only.
