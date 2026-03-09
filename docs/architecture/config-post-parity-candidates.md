# Config Post-Parity Candidates

Scope: tasks `195-197` design candidates.

## Candidate: `config explain KEY`

Intent: show resolved value, source, source path, and precedence chain.

Proposed output fields:
- `key`
- `value`
- `source`
- `source_path`
- `precedence`

## Candidate: `config doctor`

Intent: detect malformed config, duplicate keys, invalid values, and dead env overrides.

Proposed checks:
- parse validity
- duplicate key scan
- value validation scan
- env override effectiveness

## Candidate: `config schema`

Intent: improve discoverability for known keys and value rules.

Proposed output fields:
- `keys`
- `allowed_pattern`
- `value_constraints`
- `examples`

## Guardrail

All three commands are deferred until after baseline parity freeze.
