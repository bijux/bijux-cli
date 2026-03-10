# Runtime Identity Law

## Rule

`bijux` is the only canonical user-facing runtime binary name.

Internal implementation labels may exist for workspace organization and packaging boundaries, but they are not user-facing runtime identities:
- `bijux-cli-rs` is an internal crate/package concept.
- `bijux-cli-py` is an internal bridge/package concept.

## One Law, Many Entrypoints

All supported runtime entrypoints must execute the same command law:
- same command tree where parity exists
- same exit code mapping where parity exists
- same stdout/stderr envelope behavior where parity exists

The architecture principle is: one law, many entrypoints.

## Non-Negotiable Constraints

- Python bridge must execute via `bijux_cli::app::run_app`.
- Python bridge must not define independent routing rules.
- Python bridge must not define independent exit-code mapping policy.
- Python bridge must not define independent output semantics.
- Binary crate must remain an IO/process adapter, not a behavior owner.

## Enforcement

The following generated artifacts and CI gates enforce this law:
- `artifacts/status/current_rust_state.json`
- `artifacts/status/runtime_unity_report.json`
- `artifacts/parity/binary_vs_python_bridge_parity_report.json`
