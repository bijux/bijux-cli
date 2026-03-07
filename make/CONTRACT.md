# Make System Contract

## Scope
Defines the make surface for `bijux-dag`.

## Source of truth
- Root entrypoint: `Makefile`
- Make orchestration: `make/root.mk`
- Shared cargo targets: `make/cargo.mk`
- Evidence wrappers: `make/evidence.mk`
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
- Evidence targets must stay in `make/evidence.mk` and only delegate to `bijux-dev-dag`.
- Slow tests are tagged in Rust with `#[ignore = "slow"]`.
- `test` must skip ignored slow tests, `test-slow` must run only ignored tests.
- `test-all` and `coverage` must execute all tests including ignored tests.
- `evidence-all` is the canonical make entrypoint for evidence governance verification.
- `contract-all` must include evidence foundation verification via `bijux-dev-dag verify evidence-foundation`.

## Change policy
- Any target addition, rename, or removal must update `make/target-list.json`.
- Any contract-breaking surface change must update this file and `make/README.md` in the same change.
