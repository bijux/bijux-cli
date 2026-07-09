# Planner Hardening Report

## Purpose

This report records the repository surfaces that currently harden planner behavior and keep lowering claims tied to executable proof.

## Guarded surfaces

- contract: `docs/spec/PLANNER_CONTRACT.md`
- battle trust properties: `docs/spec/BATTLE_TRUST_PROPERTIES.md`
- schema: `configs/dag/schema/execution_plan.schema.json`
- trust map: `configs/dag/policy/trust_property_test_map.json`
- battle trust policy: `configs/dag/policy/battle_trust_properties.json`
- core planner tests: `crates/bijux-dag-core/tests/planner_contract.rs`, `crates/bijux-dag-core/tests/planner_fixture_contracts.rs`, `crates/bijux-dag-core/tests/planner_validation_edge_case_contracts.rs`
- runtime lowering tests: `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`, `crates/bijux-dag-runtime/tests/engine_correctness_contracts.rs`
- maintainer guard: `crates/bijux-dev/tests/planner_hardening_contracts.rs`
- maintainer command surface: `dag plan-dump`

Generated from execution-plan lowering against canonical graph fixtures in `crates/bijux-dag-core/tests/snapshots`.

## Fixture results

- `crates/bijux-dag-core/tests/snapshots/diamond.dag.json` :: nodes=`4` edges=`4` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/fan_in.dag.json` :: nodes=`3` edges=`2` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/fan_out.dag.json` :: nodes=`3` edges=`2` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/imported_bundle_replay.dag.json` :: nodes=`2` edges=`1` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/isolated_groups.dag.json` :: nodes=`3` edges=`1` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/linear.dag.json` :: nodes=`3` edges=`2` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/replay_oriented.dag.json` :: nodes=`2` edges=`1` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/resource_heavy.dag.json` :: nodes=`2` edges=`1` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/retry_heavy.dag.json` :: nodes=`2` edges=`1` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/selective_replay.dag.json` :: nodes=`4` edges=`3` stable_dump=`true` schema_required_fields=`true`

## Guardrails

- deterministic lowering across repeated runs for each fixture
- schema-required field presence from `execution_plan.schema.json`
- `tp_plan_truth` remains covered by planner lowering and engine correctness tests
- planner diagnostics such as `P4021` stay visible through `dag plan-dump`
- fixture corpus includes linear/fan/diamond/resource/retry/replay-oriented shapes
