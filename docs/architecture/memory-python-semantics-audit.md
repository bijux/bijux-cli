# Python Memory Semantics Audit

This audit captures current Python behavior from:

- `src/bijux_cli/cli/commands/memory/*.py`
- `src/bijux_cli/services/diagnostics/memory.py`
- `src/bijux_cli/cli/commands/payloads.py`

## Command surface in Python

Python currently exposes:

- `memory` (summary)
- `memory list`
- `memory get`
- `memory set`
- `memory delete`
- `memory clear`

## Storage model and path

- Backing store is JSON at `~/.bijux/.memory.json`.
- Service loads state into an in-process map and persists writes.
- Root summary and `list` are read-only operations.

## Behavior baseline for first Rust memory milestone

The parity scope in this batch is intentionally read-only:

- `memory` summary
- `memory list`

Observed baseline behavior used for Rust matching:

- Missing memory file: treated as empty state.
- Non-JSON content: tolerated as empty state in this first baseline.
- Valid JSON that is not an object: treated as malformed state error.
- Keys in `memory list` are sorted for deterministic output.

## Output and routing baseline

- Text, JSON, and YAML output modes are available and stable.
- Successful machine outputs go to stdout.
- Error envelope outputs go to stderr with non-zero exit.

## Scope decision

For 261-280, `memory` remains a built-in Rust command family, not a compatibility shim.
Write-path parity (`set/get/delete/clear`) remains outside this read-only baseline.

## Ambiguities to revisit after baseline

- Exact parse-failure handling parity for every malformed variant in Python service internals.
- Whether all write-path payload fields must match Python names before enabling write commands.
