# Replay Hardening Report

## Purpose

This report records the repository surfaces that currently harden replay
behavior and keep replay claims tied to executable proof.

## Guarded surfaces

- contract: `docs/spec/REPLAY_CONTRACT.md`
- schema: `configs/dag/schema/operator/replay_diff.schema.json`
- app tests: `crates/bijux-dag-app/tests/replay_contract.rs`
- runtime tests: `crates/bijux-dag-runtime/tests/replay_contract.rs`
- runtime invariants: `crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs`
- maintainer guard: `crates/bijux-dev/tests/replay_hardening_contracts.rs`
- evidence fixtures: `evidence/cache/replay/`
- battle scenario: `evidence/battle/workflows/replay/replay_semantic_comparison.json`

## Current hardening stance

- replay claims are allowed only when backed by the replay contract
- semantic diff mode is required in CLI command surfaces
- battle evidence must assert `replay_mandatory_proof`
- cache replay fixtures must exist for match, mismatch, corruption, and
  unsupported-version paths
- vague "replayable" documentation claims must point back to `docs/spec/REPLAY_CONTRACT.md`

## Review notes

When replay semantics, vocabulary, or evidence shape changes, update this
report together with the contract and maintainer test coverage so the human
summary and executable guards stay aligned.
