# Makefiles Contract

## Scope
Defines the role of Makefile-based wrappers in this repository.

## Authority
Makefiles are wrapper entrypoints only. Contract and governance authority remains in `crates/bijux-dev-dag`.

## Invariants
- Make targets delegate to typed control-plane commands.
- Policy, schema, release, and contract checks are implemented in Rust control-plane code.

## Related tests
- `crates/bijux-dev-dag/src/commands/mod.rs`

## Versioning and change policy
Wrapper target changes must not bypass control-plane checks.
