# CLI Contract

## Scope
Defines user-facing command behavior, command-tree stability expectations, and output mode guarantees.

## Authority
This is the normative contract for `bijux` command surfaces.

## Commands and ownership
- `dag init`: graph project initialization behavior.
- `dag validate`: graph validation only, no execution side effects.
- `dag canonicalize`: canonical serialization workflow.
- `dag lint`: static lint contract surface.
- `dag fingerprint`: fingerprint introspection output.
- `dag hash graph`: canonical graph identity alias surface.
- `dag hash artifact`: file-content artifact hash surface (`sha256`).
- `dag canonical-diff`: machine-readable raw vs canonical graph diff.
- `dag canonical-bytes`: canonical graph JSON byte emission.
- `dag run`: run orchestration and manifest emission.
- `dag replay`: replay semantics and comparability behavior.
- `dag diff`: run comparison behavior.
- `dag explain`: explain diagnostics for runs/validation.
- `dag node`: node-focused diagnostics and inspection.
- `dag status`: run status summary behavior.
- `dag verify`: artifact verification behavior.
- `dag runs history`: machine-readable ancestry listing for run directories.
- `dag runs id-explain`: run identity composition and ancestry explanation.
- `dag artifact-inspect`: artifact identity/provenance/lineage inspection surface.
- `dag cache`: cache inspection and control surfaces.
- `dag adapters`: adapter registry and capability inspection.
- `dag export`: run export bundle generation.
- `dag import`: run bundle import behavior.
- `dag version`: CLI/runtime version reporting.
- `dag doctor`: operator diagnostics surface.
- `dag migrate`: migration workflows (`migrate dag`, `migrate run`).

## Invariants
- JSON mode is machine-readable and contract-stable.
- Human-readable mode is intentionally non-contractual.
- Exit code mapping follows [ERROR_CONTRACT.md](/Users/bijan/bijux/bijux-dag/docs/spec/ERROR_CONTRACT.md).

## Related tests
- `crates/bijux-dag-app/tests/cli_contract.rs`
- `crates/bijux-dag-app/tests/output_contract.rs`
- `evidence/battle/workflows/happy_path/*`

## Versioning and change policy
Backward compatibility follows [CLI_BACKWARD_COMPATIBILITY.md](/Users/bijan/bijux/bijux-dag/docs/spec/CLI_BACKWARD_COMPATIBILITY.md). Breaking command behavior requires explicit release-note callout.
