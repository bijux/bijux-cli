# Make Layout

## Purpose
This directory defines the repository make surface with one root include and clear ownership boundaries.

## Files
- `root.mk`: public target orchestration and generated `help`.
- `cargo.mk`: cargo-native quality gates (`test`, `test-all`, `lint`, `fmt`, `check`, `audit`).
- `evidence.mk`: evidence verification wrappers delegated to `bijux-dev-dag`.
- `macros.mk`: reusable shell macros and guard helpers.
- `target-list.json`: tracked index of public targets.
- `CONTRACT.md`: rules and invariants for the make system.

## Entry flow
1. Root `Makefile` includes `make/root.mk`.
2. `make/root.mk` includes shared modules (`macros.mk`, `cargo.mk`, `evidence.mk`).
3. `help` is generated from target annotations (`##`) in loaded make files.

## Maintenance rules
- Add public targets in `make/root.mk` with a `##` description.
- Keep `make/target-list.json` synchronized with the public surface.
- Keep behavioral logic in `bijux-dev-dag`; make targets should remain wrappers.
- Keep evidence workflow wrappers in `make/evidence.mk`; do not duplicate evidence `verify` orchestration in other make files.

## Test target behavior
- `make test`: fast suite only; skips Rust tests tagged `#[ignore = "slow"]`.
- `make test-slow`: runs only tests tagged `#[ignore = "slow"]`.
- `make test-all`: runs full suite including ignored tests, then enforces release-critical evidence verification wrappers (`battle`, `cache`, `replay`, `compat`, `fault`, `perf`, `consumers`, `release-set`).
- `make coverage`: runs full suite including ignored tests.
- `make evidence-all`: runs the single evidence validation entrypoint (`bijux-dev-dag verify evidence-foundation`).
- `make evidence-verify`: runs the full evidence verification suite entrypoint.
- `make evidence-battle`: runs battle evidence verification.
- `make evidence-authoring`: runs authoring evidence verification.
- `make evidence-cache`: runs cache evidence verification.
- `make evidence-replay`: runs replay evidence verification.
- `make evidence-compat`: runs compatibility evidence verification.
- `make evidence-fault`: runs fault evidence verification.
- `make evidence-perf`: runs performance evidence verification.
- `make evidence-compare`: runs comparison evidence verification.
- `make evidence-schema`: runs schema-level evidence checks.
- `make evidence-registry`: runs registry integrity and drift checks.
- `make evidence-consumers`: runs consumer integrity and bypass checks.
- `make evidence-release-set`: validates release evidence set references.
- `make evidence-report`: generates machine-readable and human-readable evidence suite summary reports.
- `make evidence-clean`: cleans generated evidence artifacts.
- `make contract-all`: runs all contracts plus evidence foundation verification.
