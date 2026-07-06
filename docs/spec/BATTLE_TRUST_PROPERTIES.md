# Battle Trust Properties

## Scope

This contract defines the trust properties that battle workflows protect, how
those properties map to executable scenarios, and which policy files remain
authoritative for release-blocking battle evidence.

## Authoritative inputs

The following surfaces are authoritative for battle trust-property governance:

- `configs/dag/policy/battle_trust_properties.json`
- `configs/dag/policy/trust_property_test_map.json`
- `configs/dag/policy/battle_release_blocking_subset.json`
- `evidence/battle/metadata.json`
- `evidence/battle/registries/trust_property_registry.json`

## Trust property model

Battle trust properties express the release-critical runtime truths that must
stay covered by executable workflow evidence. Each property must have:

- a stable `id`
- a human-readable name and description
- scenario coverage in battle metadata and trust maps
- executable test coverage in the trust-property test map

The registry must include `tp_plan_truth` so planner and lowering correctness
remain part of battle evidence instead of being treated as a schema-only claim.

## Scenario mapping rules

- every required battle scenario must exist as evidence and metadata
- every scenario in `scenario_trust_map` must map to declared trust properties
- release-top trust properties must be protected by at least one mapped scenario
- advisory scenarios must not silently become release-blocking

## Planner and lowering relationship

`tp_plan_truth` covers the contract that semantically equivalent graphs lower to
equivalent execution plans and that unsupported runtime requirements fail
during planning with explicit diagnostics such as `P4021`.

Planner-facing documentation may describe plan truth or lowering determinism as
release-hardening evidence only when it cites this document and
`docs/spec/PLANNER_CONTRACT.md` directly.

## Related tests

- `crates/bijux-dev/tests/battle_suite_concentration_contracts.rs`
- `crates/bijux-dev/tests/planner_hardening_contracts.rs`
- `crates/bijux-dag-runtime/tests/battle_workflow_harness_contracts.rs`
- `crates/bijux-dag-runtime/tests/planner_lowering_contracts.rs`

## Versioning and change policy

Trust property IDs, scenario mapping rules, and planner-related trust semantics
are stable contract surfaces. Any incompatible change requires updating this
document, the authoritative policy files, and the linked battle and planner
contract tests in the same change.
