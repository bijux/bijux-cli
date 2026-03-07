# Make System Contract

## Scope
Defines the make surface for `bijux-dag`.

## Source of truth
- Root entrypoint: `Makefile`
- Make orchestration: `make/root.mk`
- Shared cargo targets: `make/cargo.mk`
- Shared helpers: `make/macros.mk`
- Tracked public target index: `make/target-list.json`

## Authority boundaries
- Makefiles are wrapper and orchestration surfaces only.
- Behavioral authority for repository governance remains in `crates/bijux-dev-dag`.
- Rust workflow authority remains in cargo commands and workspace crate code.

## Invariants
- `Makefile` must only include `make/root.mk`.
- Public targets must be listed in `make/target-list.json`.
- `help` output must be generated from annotated targets (`##`) and must not be hand-coded.
- Checks, contracts, release, and repository governance execution paths must route through `bijux-dev-dag`.
- Cargo-native gates (`test`, `test-all`, `lint`, `fmt`, `check`, `audit`) must stay in `make/cargo.mk`.

## Change policy
- Any target addition, rename, or removal must update `make/target-list.json`.
- Any contract-breaking surface change must update this file and `make/README.md` in the same change.
