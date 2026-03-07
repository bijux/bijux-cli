# Diagnostics Modes

## Scope
Defines boundaries between default diagnostics and debug diagnostics.

## Modes
- `default`: user-facing summary with stable code/category and actionable hint when available.
- `debug`: includes internal context, source-chain data, and policy trace data.

## Boundaries
- Internal implementation details are hidden in default mode.
- Debug mode may include crate/module origin and underlying source errors.

## Related tests
- `crates/bijux-dag-app/tests/error_output_contract.rs`

## Versioning and change policy
New debug fields are additive. Removing existing default fields is breaking.
