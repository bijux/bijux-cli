# History Parity Report

Scope: tasks 241-260 (first read-only history parity milestone).

## Implemented

- Root `history` routed through Rust core with real file-backed reads.
- History path resolution uses shared compatibility discovery path flow.
- JSON-array history parsing with deterministic behavior:
  - missing file -> `entries: []`
  - non-array JSON -> error exit path
  - non-object entries in array -> skipped
- Compatibility fallback for line-oriented history files, including malformed-line tolerance.
- Root read filters added for parity coverage:
  - `--limit`
  - `--filter`
  - `--sort timestamp`

## Tests added

- Binary history parity tests:
  - format snapshots (text/json/yaml)
  - missing/malformed behavior and exit routing
  - huge history file tail-limit behavior
  - duplicate entry retention and ordering
  - REPL line-history + CLI interop
  - Python-vs-Rust read-only parity comparison

Files:

- `crates/bijux-cli/tests/cli_surface/history/history_parity.rs`
- `crates/bijux-cli/tests/cli_surface/snapshots/history_root_json.txt`
- `crates/bijux-cli/tests/cli_surface/snapshots/history_root_yaml.txt`
- `crates/bijux-cli/tests/cli_surface/snapshots/history_root_text.txt`

## Status for 241-260

- `241`: complete (audit doc)
- `242-245`: complete for first read-only command and resilient parsing baseline
- `246-249`: complete (empty/missing/malformed/huge cases)
- `250`: complete for read-output truncation behavior (tail limiting)
- `251-252`: complete (duplicate and ordering tests)
- `253`: complete (REPL line format and CLI read interop test)
- `254`: complete (`BIJUXCLI_HISTORY_FILE` override test)
- `255-257`: complete (text/json/yaml snapshots)
- `258`: complete (Python-vs-Rust parity test for first history command)
- `259`: complete (failure exit + stdout/stderr tests)
- `260`: complete (this report)

## Remaining gaps

- History write-path parity (clear/import/export mutation semantics).
- Cross-process lock parity assertions against Python write path.
- Full grouped summary parity (`--group-by`) for Rust history root is not yet implemented.
