# Configs Contract

## Scope
`configs/` contains authoritative machine-readable policy, schema, and environment configuration inputs used by developer tooling and runtime validation.

## Authority
`configs/` is authoritative for repository policy inputs when a corresponding code path explicitly points to a file under this tree.

## Invariants
- Policy SSOT files live under `configs/policy/`.
- Schemas live under `configs/schema/`.
- Changes to authoritative files must update linked docs and guards.

## Allowed changes
- Additive schema/policy evolution consistent with compatibility contracts.
- New config domains with documented owner and validator.

## Related tests
- `crates/bijux-dev-dag/src/commands/mod.rs` repo/doc guards
- `crates/bijux-dev-dag/src/suites/repo.rs`

## Related schemas
- `configs/schema/*.schema.json`
- `configs/policy/dependency_rules.json`
- `configs/policy/source_layout.json`

## Versioning and change policy
Authoritative config contracts require explicit version fields or compatibility policy linkage when format changes are introduced.
