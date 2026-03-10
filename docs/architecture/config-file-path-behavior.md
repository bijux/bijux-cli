# Config File And Path Behavior

This document records decisions and coverage for tasks 81-100.

## File format baseline

- Rust config storage uses dotenv-style `KEY=VALUE` lines.
- Parser accepts blank lines and comment lines beginning with `#`.
- Parser rejects malformed lines without `=`.
- Duplicate keys use last-write-wins semantics.

## Comments and formatting behavior

- Comment lines are accepted during parse.
- Comments are not preserved on write.
- Original whitespace formatting is not preserved on write.
- Output ordering is deterministic from normalized key ordering.

These decisions intentionally prioritize deterministic machine behavior and parity-stable replay over textual formatting fidelity.

## Path behavior baseline

- Default config path: `~/.bijux/.env`.
- Environment override: `BIJUXCLI_CONFIG`.
- CLI flag override: `--config-path` takes precedence over environment path.

## Error behavior baseline

- Missing config file on read returns empty map behavior.
- Unreadable config file returns error result.
- Unwritable target path returns error result.
- Missing parent directory on write is created automatically.

## Coverage map

- Parser/write behavior tests: `crates/bijux-cli-core/src/config/storage.rs` unit tests.
- Path precedence tests: `crates/bijux-cli-bin/tests/config_parity.rs`.
- Unreadable/unwritable path tests: `crates/bijux-cli-core/tests/config_parity.rs`.
- Default/env path resolution tests: `crates/bijux-cli-core/src/install/mod.rs`.
