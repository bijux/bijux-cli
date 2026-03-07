# CLI Contract

## Scope
Defines user-facing command behavior, command-tree stability expectations, and output mode guarantees.

## Authority
This is the normative contract for `bijux` command surfaces.

## Commands and ownership
- `dag run`: run orchestration and manifest emission.
- `dag validate`: graph validation only, no execution side effects.
- `dag replay`: replay semantics and comparability behavior.
- `dag cache`: cache inspection and control surfaces.
- `dag export`: run export bundle generation.
- `dag import`: run bundle import behavior.
- `dag graph`: graph rendering/introspection outputs.
- `dag explain`: explain diagnostics for runs/validation.
- `dag status`: run status summary behavior.

## Invariants
- JSON mode is machine-readable and contract-stable.
- Human-readable mode is intentionally non-contractual.
- Exit code mapping follows [ERROR_CONTRACT.md](/Users/bijan/bijux/bijux-dag/docs/spec/ERROR_CONTRACT.md).

## Related tests
- `crates/bijux-dag-app/tests/cli_contract.rs`
- `crates/bijux-dag-app/tests/output_contract.rs`
- `tests/e2e/happy_path/*`

## Versioning and change policy
Backward compatibility follows [CLI_BACKWARD_COMPATIBILITY.md](/Users/bijan/bijux/bijux-dag/docs/spec/CLI_BACKWARD_COMPATIBILITY.md). Breaking command behavior requires explicit release-note callout.
