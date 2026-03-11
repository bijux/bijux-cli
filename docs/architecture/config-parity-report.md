# Config Parity Report

Scope: stable parity and behavior coverage. (`cli config get` and `cli config set` parity hardening and validation coverage).

## Implemented in Rust

- `cli config get KEY` now:
  - Normalizes key (`BIJUXCLI_` prefix optional, case-insensitive lookup).
  - Applies env override before file lookup.
  - Returns deterministic not-found error with exit `2`.
  - Supports `--config-path` override (flag precedence over env).
- `cli config set KEY=VALUE` now:
  - Parses and validates `KEY=VALUE` with Python-aligned rules.
  - Rejects invalid keys, unknown dotted sections, non-ASCII, and control chars.
  - Creates parent directories when missing.
  - Writes atomically and preserves unrelated keys.
  - Repeated writes are idempotent for the same key/value.
- Failure and stream routing:
  - Successful machine output stays on stdout with empty stderr.
  - Error envelopes route to stderr with empty stdout.

## Added Test Coverage

- Core-level parity tests: `crates/bijux-cli/tests/integration/cli/config/config_parity.rs`.
- Binary-level parity tests: `crates/bijux-cli/tests/integration/cli/config/config_parity.rs`.
- Python-vs-Rust compatibility tests: `crates/bijux-cli/tests/integration/cli/config/config_python_compatibility.rs`.

Covered scenarios include:

- deterministic reads for missing files/keys
- malformed file read behavior
- env and flag config-path overrides
- env value precedence over file value
- text/json/yaml serialization for `config get`
- idempotent repeated writes
- preservation of unrelated settings
- stderr/stdout routing on success/failure

## Known Gaps

- Python `config set` stdin fallback (no-argument mode) is not yet enabled in Rust.
- Cross-process lock contention parity is not yet modeled in Rust config writes.
- `load` and `export` parity are now covered in the export/load parity report.

## Status (221-240)

- `221-223`: complete for audited `get`/`set` semantics.
- `224-231`: complete for deterministic behavior, validation, serialization, preservation, and idempotence.
- `232`: partial (contention behavior documented; lock parity not yet implemented).
- `233-238`: complete for malformed/partial reads and env/flag overrides.
- `239`: complete with compatibility tests against Python command outputs.
- `240`: complete (this report).
