# Planner Hardening Report

Generated from execution-plan lowering against canonical graph fixtures in `crates/bijux-dag-core/tests/snapshots`.

## Fixture results

- `crates/bijux-dag-core/tests/snapshots/diamond.dag.json` :: nodes=`4` edges=`4` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/fan_in.dag.json` :: nodes=`3` edges=`2` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/fan_out.dag.json` :: nodes=`3` edges=`2` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/isolated_groups.dag.json` :: nodes=`3` edges=`1` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/linear.dag.json` :: nodes=`3` edges=`2` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/replay_oriented.dag.json` :: nodes=`2` edges=`1` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/resource_heavy.dag.json` :: nodes=`2` edges=`1` stable_dump=`true` schema_required_fields=`true`
- `crates/bijux-dag-core/tests/snapshots/retry_heavy.dag.json` :: nodes=`2` edges=`1` stable_dump=`true` schema_required_fields=`true`

## Guardrails

- deterministic lowering across repeated runs for each fixture
- schema-required field presence from `execution_plan.schema.json`
- fixture corpus includes linear/fan/diamond/resource/retry/replay-oriented shapes
