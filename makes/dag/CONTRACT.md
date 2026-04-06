# Make System Contract

## Scope
Defines the make surface for `bijux-dag`.

## Source of truth
- Root entrypoint: `Makefile`
- Make orchestration: `makes/dag/root.mk`
- Shared cargo targets: `makes/dag/cargo.mk`
- Evidence wrappers: `makes/dag/evidence.mk`
- Shared helpers: `makes/dag/macros.mk`
- Tracked public target index: `makes/dag/target-list.json`

## Authority boundaries
- Makefiles are wrapper and orchestration surfaces only.
- Behavioral authority for repository governance remains in `crates/bijux-core-dev`.
- Rust workflow authority remains in cargo commands and workspace crate code.

## Invariants
- `Makefile` must only include `makes/dag/root.mk`.
- Public targets must be listed in `makes/dag/target-list.json`.
- `help` output must be generated from annotated targets (`##`) and must not be hand-coded.
- Checks, contracts, release, and repository governance execution paths must route through `bijux-dev-dag`.
- Cargo-native gates (`test`, `test-all`, `lint`, `fmt`, `check`, `audit`) must stay in `makes/dag/cargo.mk`.
- Evidence targets must stay in `makes/dag/evidence.mk` and only delegate to `bijux-dev-dag`.
- Slow tests are tagged in Rust with `#[ignore = "slow"]`.
- `test` must skip ignored slow tests, `test-slow` must run only ignored tests.
- `test-all` and `coverage` must execute all tests including ignored tests.
- `evidence-all` is the canonical make entrypoint for evidence governance verification.
- `contract-all` must include evidence foundation verification via `bijux-dev-dag verify evidence-foundation`.

## Change policy
- Any target addition, rename, or removal must update `makes/dag/target-list.json`.
- Any contract-breaking surface change must update this file and `makes/dag/README.md` in the same change.
