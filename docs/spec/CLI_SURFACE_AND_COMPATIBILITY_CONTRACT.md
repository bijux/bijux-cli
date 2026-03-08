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
