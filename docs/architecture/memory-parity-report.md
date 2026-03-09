# Memory Parity Report

Scope: tasks 261-280 (first memory baseline).

## Implemented in Rust

- Routed built-in commands:
  - `memory`
  - `memory list`
- Root and list execution implemented in core (no temporary shim path).
- Read path resolves from HOME-based memory file (`~/.bijux/.memory.json`).
- Machine-visible contracts added:
  - `MemorySummary`
  - `MemoryKeyList`

## Behavior and parity checks

- Empty state behavior:
  - missing file -> success with empty result
- Malformed state behavior:
  - non-JSON bytes -> treated as empty baseline state
  - valid non-object JSON -> normalized error envelope on stderr
- Output modes covered:
  - text
  - json (pretty and compact)
  - yaml
- Stream routing covered:
  - success payloads on stdout
  - failure envelope on stderr
- Exit behavior covered:
  - success -> `0`
  - malformed non-object JSON -> `1`

## Tests added

- `crates/bijux-cli-bin/tests/memory_parity.rs`
- `crates/bijux-cli-bin/tests/snapshots/memory_list_text.txt`
- `crates/bijux-cli-bin/tests/snapshots/memory_list_yaml.txt`
- `crates/bijux-cli-core/tests/app_direct_invocation.rs` memory cases
- `crates/bijux-cli-contracts/tests/serde_roundtrip.rs` memory contract roundtrips

## Status for 261-280

- `261`: complete (Python audit documented)
- `262`: complete (`memory` stays built-in)
- `263`: complete (`memory list` read-only command ported)
- `264`: complete (memory payload contracts defined)
- `265-267`: complete (empty/missing/malformed tests)
- `268-270`: complete (text/json/yaml output coverage)
- `271`: complete (quiet mode coverage)
- `272`: complete (error normalization coverage)
- `273`: complete (config/env interaction coverage)
- `274`: complete (`--pretty` handling coverage)
- `275`: complete (Python-vs-Rust parity test for summary)
- `276`: complete (stderr/stdout failure routing test)
- `277`: complete (failure exit-code test)
- `278`: complete (this report)
- `279`: complete (still Python-only: `memory get`, `memory set`, `memory delete`, `memory clear`)
- `280`: complete (first read-only memory baseline frozen in this report)

## Remaining memory gaps after this baseline

- Write-path parity for `set/get/delete/clear`.
- Full Python payload-field parity for write-path responses.
- Additional parity captures for Python `memory list` in complex malformed inputs.
