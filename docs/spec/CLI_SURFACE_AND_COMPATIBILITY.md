# CLI SURFACE AND COMPATIBILITY

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/BIJUX_CLI_INTEGRATION_CONTRACT.md
# Bijux CLI Integration Contract

## Purpose
Define boundaries between root `bijux` command surfaces and `bijux dag` semantics.

## Command ownership
- `bijux dag` owns DAG semantics and runtime truth surfaces.
- root `bijux` may compose orchestration UX but must not alter DAG identity/replay semantics.

## Integration rule
All composed CLI surfaces must preserve `bijux-dag` JSON contracts and exit semantics.

## SOURCE: docs/spec/CLI_BACKWARD_COMPATIBILITY.md
# Superseded by CLI cluster contract

- Superseded by: [CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md](./CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md)
- Appendix source: [appendices/cli/CLI_BACKWARD_COMPATIBILITY.md](./appendices/cli/CLI_BACKWARD_COMPATIBILITY.md)

## SOURCE: docs/spec/CLI_COMMAND_STABILITY_DOCUMENTATION.md
# Superseded by CLI cluster contract

- Superseded by: [CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md](./CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md)
- Appendix source: [appendices/cli/CLI_COMMAND_STABILITY_DOCUMENTATION.md](./appendices/cli/CLI_COMMAND_STABILITY_DOCUMENTATION.md)

## SOURCE: docs/spec/CLI_CONTRACT.md
# Superseded by CLI cluster contract

- Superseded by: [CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md](./CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md)
- Appendix source: [appendices/cli/CLI_CONTRACT.md](./appendices/cli/CLI_CONTRACT.md)

## SOURCE: docs/spec/CLI_DEPRECATION_AND_ALIAS_POLICY.md
# Superseded by CLI cluster contract

- Superseded by: [CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md](./CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md)
- Appendix source: [appendices/cli/CLI_DEPRECATION_AND_ALIAS_POLICY.md](./appendices/cli/CLI_DEPRECATION_AND_ALIAS_POLICY.md)

## SOURCE: docs/spec/CLI_OWNERSHIP.md
# Superseded by CLI cluster contract

- Superseded by: [CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md](./CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md)
- Appendix source: [appendices/cli/CLI_OWNERSHIP.md](./appendices/cli/CLI_OWNERSHIP.md)

## SOURCE: docs/spec/CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md
# CLI surface and compatibility contract

**What this spec is not**: user onboarding walkthrough, changelog formatting guide, or internal release process.

## Scope

This contract is the single source of truth for:

- command tree and stable surface expectations
- JSON and text output contract intent
- alias and deprecation behavior
- ownership split for command composition vs command semantics

## Stable guarantees

- Top-level command names and stable subcommands remain stable once documented.
- JSON envelope keys and failure classes are stable unless explicitly superseded with migration plan.
- New flags and additive fields are preferred over behavior repurposing.
- Removal of stable commands/fields requires compatibility decision and migration notes.

## Governance

- Command taxonomy and compatibility updates update both `docs/reference/COMMAND_TAXONOMY.md` and relevant contract evidence.
- Breaking changes to command behavior are prohibited without release-ready migration surface.

## Evidence and implementation links

- CLI surface behavior: `crates/bijux-dag-cli` and `crates/bijux-dag-app`.
- Contract suites: `crates/bijux-dag-app/tests/cli_contract.rs`, `crates/bijux-dag-cli/tests`.
- JSON envelope schema checks and governance suites under `crates/bijux-dev-dag`.

## Canonical appendices

- [cli command stability](./appendices/cli/CLI_COMMAND_STABILITY_DOCUMENTATION.md)
- [cli backward compatibility](./appendices/cli/CLI_BACKWARD_COMPATIBILITY.md)
- [surface stability policy](./appendices/cli/CLI_SURFACE_STABILITY_POLICY.md)
- [deprecation and aliases](./appendices/cli/CLI_DEPRECATION_AND_ALIAS_POLICY.md)
- [ownership](./appendices/cli/CLI_OWNERSHIP.md)
- [control-plane command taxonomy](./appendices/cli/CONTROL_PLANE_COMMAND_TAXONOMY.md)

## Superseded paths

Legacy root spec files now point to this cluster:
- `CLI_CONTRACT.md`

## SOURCE: docs/spec/CLI_SURFACE_STABILITY_POLICY.md
# Superseded by CLI cluster contract

- Superseded by: [CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md](./CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md)
- Appendix source: [appendices/cli/CLI_SURFACE_STABILITY_POLICY.md](./appendices/cli/CLI_SURFACE_STABILITY_POLICY.md)

## SOURCE: docs/spec/CONTROL_PLANE_COMMAND_TAXONOMY.md
# Superseded by CLI cluster contract

- Superseded by: [CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md](./CLI_SURFACE_AND_COMPATIBILITY_CONTRACT.md)
- Appendix source: [appendices/cli/CONTROL_PLANE_COMMAND_TAXONOMY.md](./appendices/cli/CONTROL_PLANE_COMMAND_TAXONOMY.md)

## SOURCE: docs/spec/appendices/cli/CLI_BACKWARD_COMPATIBILITY.md
# CLI backward compatibility policy

## Contract surface

Only the command tree and JSON envelope are compatibility contracts.
Human-readable plaintext output is intentionally non-contractual.

## Stable guarantees

- Top-level command names are stable once documented in `docs/reference/COMMAND_TAXONOMY.md`.
- JSON envelope shape (`ok`, `command`, `data`, `diagnostics`) is stable.
- Documented non-zero exit code classes remain stable.

## Allowed changes

- Additive JSON fields under `data`.
- New subcommands that do not change existing command semantics.
- Improved plaintext wording and formatting.

## Breaking changes

- Removing or renaming documented command names.
- Changing JSON envelope shape.
- Reassigning established non-zero exit code classes.

## Governance

Any breaking CLI change requires a compatibility decision record and migration notes.

## SOURCE: docs/spec/appendices/cli/CLI_COMMAND_STABILITY_DOCUMENTATION.md
# CLI Command Stability Documentation

## Stable Command Families

- graph authoring and validation: `validate`, `hash graph`, `fingerprint`
- execution and replay: `run`, `replay`, `diff`, `why-rerun`, `why-cache-missed`
- run introspection: `runs list/show/inspect/history/timeline/tree/id-explain/explain-failure`
- artifact introspection: `artifact-inspect`, `trace-artifact`
- verification and portability: `prove`, `verify`, `export`, `import`, `fsck`

## Contract Anchors

- CLI surface tests: `crates/bijux-dag-cli/tests`
- JSON schema lockstep tests: `crates/bijux-dev-dag/tests/json_output_governance_contracts.rs`
- Human-output snapshots: `crates/bijux-dag-app/tests/snapshots`
- Error code policy: `configs/policy/error_codes.json`

## Compatibility Intent

- Additive flags are preferred over behavioral redefinition.
- Existing flags are not repurposed.
- Stable commands preserve machine-readable JSON contracts across patch releases.

## SOURCE: docs/spec/appendices/cli/CLI_CONTRACT.md
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
- `dag hash run`: run identity hash surface from durable run records.
- `dag hash artifact`: file-content artifact hash surface (`sha256`).
- `dag canonical-diff`: machine-readable raw vs canonical graph diff.
- `dag canonical-bytes`: canonical graph JSON byte emission.
- `dag run`: run orchestration and manifest emission.
- `dag replay`: replay semantics and comparability behavior.
- `dag prove`: proof bundle generation and completeness reporting surface.
- `dag proof-summary`: operator-facing concise proof summary surface.
- `dag diff`: run comparison behavior.
- `dag explain`: explain diagnostics for runs/validation.
- `dag node`: node-focused diagnostics and inspection.
- `dag status`: run status summary behavior.
- `dag verify`: artifact verification behavior.
- `dag fsck`: alias verification surface for run integrity inspection.
- `dag runs history`: machine-readable ancestry listing for run directories.
- `dag runs id-explain`: run identity composition and ancestry explanation.
- `dag artifact-inspect`: artifact identity/provenance/lineage inspection surface.
- `dag replay --dry-run`: replay planning surface without execution side effects.
- `dag replay --prove`: replay fidelity proof output surface.
- `dag why-rerun`: root cause summary for semantic replay divergence.
- `dag why-cache-missed`: cache eligibility and miss reason summary.
- `dag trace-artifact`: lineage-aware artifact trace surface.
- `dag cache`: cache inspection and control surfaces.
- `dag adapters`: adapter registry and capability inspection.
- `dag export`: run export bundle generation.
- `dag export --from-run`: source-explicit export alias for run directory bundles.
- `dag export --without-artifacts`: metadata-only bundle export that excludes artifact payload and indexes.
- `dag export --provenance-only`: provenance evidence bundle export without node trace/output payloads.
- `dag export --redact`: privacy-safe bundle export with redacted sensitive provenance fields.
- `dag import`: run bundle import behavior.
- `dag import --verify-only`: bundle verification-only path without runtime mutation.
- `dag version`: CLI/runtime version reporting.
- `dag doctor`: operator diagnostics surface.
- `dag migrate`: migration workflows (`migrate dag`, `migrate run`).
- `dag capabilities --backend kubernetes`: backend-specific capability report surface.
- `dag capabilities --backend hpc`: backend-specific capability report surface.
- `dag capabilities --backend remote`: backend-specific capability report surface.
- `dag semantic-portability --backend <name>`: backend-target portability and downgrade report surface.
- `dag equivalence-proof <run-a> <run-b> --backend-a <name> --backend-b <name>`: cross-backend run equivalence proof report surface.
- `dag version-inspect --dag|--run-dir|--export-bundle`: schema/format inspection surface.
- `dag migrate dag|run --from <v> --to <v> --dry-run`: schema migration preview surface.

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

## SOURCE: docs/spec/appendices/cli/CLI_DEPRECATION_AND_ALIAS_POLICY.md
# CLI Deprecation and Alias Policy

## Scope

Defines how command aliases and deprecations are introduced and governed for `bijux`.

## Alias rules

- Aliases must be explicit and documented in command taxonomy and CLI contract docs.
- Aliases must preserve exit-code class and JSON envelope compatibility.
- Aliases must not silently narrow validation behavior.

## Deprecation rules

- A deprecated command must keep working for at least one documented compatibility window.
- Deprecation requires:
  - replacement command
  - migration note
  - release-note entry
  - contract test coverage
- Deprecated surfaces must emit stable diagnostics in both human and JSON modes.

## Current aliases

- `dag fsck <run-dir>` is a stable alias surface for run integrity verification (`dag verify <run-dir> --deep`).

## Tests

- `crates/bijux-dag-cli/tests/contract_surface.rs`
- `crates/bijux-dag-app/tests/cli_contract.rs`

## SOURCE: docs/spec/appendices/cli/CLI_OWNERSHIP.md
# CLI ownership boundaries

`bijux-cli` owns entrypoint composition and shell-level command wiring.

`bijux-dag` owns DAG command semantics, output envelopes, exit behavior, and compatibility guarantees.

## Responsibility split

- `crates/bijux-dag-cli`:
  - top-level `bijux` command tree
  - sub-app mounting (`dag`)
  - completions surface
- `crates/bijux-dag-app`:
  - `dag` command behavior and routing
  - JSON and text response contracts
  - legacy alias behavior (`dag status`, `dag verify`, `dag diff`)

## Change policy

- DAG semantics changes must land in `bijux-dag-app` with contract tests.
- `bijux-dag-cli` may not implement runtime semantics.
- Command taxonomy updates must update:
  - `docs/reference/COMMAND_TAXONOMY.md`
  - `docs/CLI.md`
  - CLI contract tests in `crates/bijux-dag-cli/tests`.

## SOURCE: docs/spec/appendices/cli/CLI_SURFACE_STABILITY_POLICY.md
# CLI Surface Stability Policy

## Scope

This policy governs the public `bijux dag` command surface, including flags, exit codes, JSON envelopes, and human-readable output expectations.

## Stability Rules

1. Existing top-level commands and documented aliases are stable by default.
2. JSON output envelopes must keep `command`, `status`, and `data` fields stable.
3. Exit code semantics must remain aligned with `configs/policy/error_codes.json`.
4. Help output may evolve wording, but command/flag presence must remain backward-compatible for stable surfaces.
5. Deprecated surfaces require documented migration guidance before removal.

## Change Requirements

- Any CLI-breaking change requires:
  - updated docs in `docs/spec`
  - regression tests in `crates/bijux-dag-cli/tests`
  - release note entry

## Non-Goals

- Freezing all help-text phrasing byte-for-byte
- Guaranteeing unchanged stderr wording for internal errors

## SOURCE: docs/spec/appendices/cli/CONTROL_PLANE_COMMAND_TAXONOMY.md
# Control-plane command taxonomy

## Repo verification

- `bijux-dev-dag repo run`
- `bijux-dev-dag repo list`
- `bijux-dev-dag repo explain --suite <id>`

## Docs verification

- `docs-governance`
- `docs-links`
- `docs-schema-ref`
- `docs-contract-ref`

## Naming verification

- `naming-governance`

## Crate boundary verification

- `crate-boundary-foundation`
- `dep-guard`

## Fixture and artifact verification

- `test-trust-foundation`
- `artifact-hardening`
- `run-dir-audit --run-dir <path> [--strict]`

## Release and ci verification

- `release verify`
- `release post-release-verify`
- `ci`
- `repo-hygiene-suite`
- `foundation-review-report`
