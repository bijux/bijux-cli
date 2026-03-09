# Python History Semantics Audit

This audit captures current Python behavior from:

- `src/bijux_cli/cli/commands/history/service.py`
- `src/bijux_cli/services/history/__init__.py`

## Root `history` behavior

Default command (`bijux history`) lists entries and returns payload:

- `{ "entries": [...] }`
- Default `limit` is `20` and returns most recent entries.

Supported read modifiers in root command:

- `--limit/-l`: non-negative; `0` returns `[]`.
- `--filter/-F`: keeps entries whose `command` contains substring.
- `--sort timestamp`: sorts by timestamp.
- `--group-by command`: grouped summary mode.

## History file resolution

Python service resolves history path with precedence:

1. explicit constructor path
2. `BIJUXCLI_HISTORY_FILE`
3. `<BIJUXCLI_CONFIG parent>/.bijux_history`
4. default `~/.bijux/.history`

## File format and resilience

Primary format is JSON array of history objects.

- Missing file: treated as empty list.
- Empty file: treated as empty list.
- Corrupted JSON / non-array JSON: captured as load error; list command fails.
- Non-dict entries in JSON array are ignored.
- `command` field is ASCII-cleaned in service load path.

## Concurrency and persistence

Python history writes are cross-process coordinated with lock files on POSIX and atomic replace writes.
Root list operation is read-only and does not lock.

## Baseline scope for Rust parity batch

This batch targets first read-only parity only (`history` root list semantics and path/format resilience).
Write-path commands (`history clear`, import/export mutations) remain out of this batch.
