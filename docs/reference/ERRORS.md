# Error Reference

This reference is machine-oriented and aligned with [ERROR_CONTRACT](../spec/ERROR_CONTRACT.md) and [ERROR_TAXONOMY](../spec/ERROR_TAXONOMY.md).

## Stable fields
- `category`
- `code`
- `message`
- `command`
- `exit_code`

## Output modes
- JSON mode includes structured error fields.
- Human mode is concise and remediation-oriented; internal debug context is excluded unless debug mode is enabled.

## Verbosity model
- Default: cause + action only.
- Debug: include internal context and underlying source chain.

## Remediation hints
Hints are emitted only when mechanically justified by known rule IDs or invariant checks.
