# Config Set Parity Report

Scope: `config set` command parity baseline and reliability checks.

## Implemented Behavior

- Accepts direct `KEY=VALUE` input.
- Accepts stdin fallback when `KEY=VALUE` is omitted and stdin is non-terminal.
- Preserves Python-aligned key/value validation and normalization rules.
- Persists with deterministic ordering and atomic replace behavior.
- Keeps success payloads on stdout and failure payloads on stderr.

## Coverage

- Binary command coverage: `crates/bijux-cli/tests/integration/cli/config/config_set_parity.rs`
- Core command coverage: `crates/bijux-cli/tests/integration/cli/config/config_parity.rs`
- Key/value rule coverage: `crates/bijux-cli/tests/integration/cli/config/config_key_value_parity.rs`
- Python compatibility coverage: `crates/bijux-cli/tests/integration/cli/config/config_python_compatibility.rs`

## Captured Outputs

- Text snapshot: `crates/bijux-cli/tests/data/golden/cli_surface/config_set_text.txt`
- JSON pretty snapshot: `crates/bijux-cli/tests/data/golden/cli_surface/config_set_json_pretty.txt`
- JSON compact snapshot: `crates/bijux-cli/tests/data/golden/cli_surface/config_set_json_compact.txt`
- YAML snapshot: `crates/bijux-cli/tests/data/golden/cli_surface/config_set_yaml_pretty.txt`

## Status Matrix

- `set` direct pair parsing: complete
- stdin fallback parity: complete
- quoted and escaped value parsing parity: complete
- create and overwrite behavior: complete
- unrelated key preservation: complete
- deterministic file ordering: complete
- disk-write failure envelope: complete
- original-file retention on write failure: complete
- Python exit/stream parity: complete

## Deferred Improvements

See `docs/architecture/config-set-post-parity-improvements.md`.
