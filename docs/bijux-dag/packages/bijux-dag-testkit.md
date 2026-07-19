---
title: bijux-dag-testkit Package
audience: maintainers
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-19
---

# bijux-dag-testkit

`bijux-dag-testkit` is repository-internal support for deterministic tests
shared across the DAG workspace. It is `publish = false` and is not part of the
public `v0.4.0` crates.io package family.

Use it to remove duplicated fixture construction or evidence assertions. Do
not move production semantics into it and do not use a helper result as a
substitute for exercising the boundary under test.

## Owned Surfaces

| Module | Provides | Does not prove |
| --- | --- | --- |
| `workflows` | graph builders, graph fixtures, normalized run snapshots | runtime scheduling or adapter execution |
| `fake_adapter` | deterministic output and failure scenarios | process, container, network, or timeout enforcement |
| `product_scenarios` | validation of supplied scenario reports | that the represented product workflow actually ran |
| crate root | evidence readers, registry lookup, graph shapes, manifest and trace assertions | domain correctness of retained output |

The crate depends on `bijux-dag-core` and `bijux-dag-artifacts` to build and
inspect test material. Production DAG crates do not depend on the testkit.
Repository maintainer tests consume it as a development dependency.

## Graph Builders

`DagFixture` builds small graphs through the same core domain types used by the
product. Shared fixtures cover chains, diamonds, fan-out, disconnected graphs,
retry, timeout, cache, replay, branch/join, map/reduce, container, external
adapter, and failure shapes.

Use a shared graph when multiple packages must agree on the same topology.
Keep a fixture local when it exists to test one package-specific parser,
serializer, or failure. Centralizing every test input would blur ownership and
make unrelated suites change together.

## Evidence Access

The testkit can load text, JSON, typed fixtures, and assets from the governed
evidence registry. Checked registry functions return actionable errors for
missing or malformed evidence; prefer them when a test needs to assert refusal
behavior.

New cross-package tests should:

1. register durable evidence under `evidence/dag/`;
2. resolve it by governed asset identity where consumer tracking matters;
3. use canonical `evidence/dag/...` paths when direct paths are appropriate;
4. update the registry consumer map in the same change.

Legacy path remapping remains for existing consumers. It is compatibility
support, not the canonical path for new tests.

## Snapshot Discipline

`collect_run_dir_snapshot` captures file inventory, node traces, indexes, and
retained payloads. Snapshot normalization replaces nondeterministic timestamps,
process IDs, and captured tool version. It intentionally leaves statuses,
identities, digests, paths, failures, and payload content visible.

Snapshot updates require review of every semantic difference. Do not expand
normalization to make a changing contract appear stable.

## Fake Adapter Boundary

`FakeAdapterHarness` materializes deterministic scenarios:

- successful output;
- execution failure;
- timeout-shaped failure;
- missing required output;
- corrupt bytes;
- large output;
- harness error.

The timeout scenario returns timeout-shaped evidence; it does not wait on or
terminate a real process. The corrupt-output scenario produces unusual bytes;
artifact verification still must be exercised by the owning artifacts or
runtime test. Real backend claims require real boundary tests.

## Scenario Reports

The builders in `product_scenarios` accept a report and reject it when required
proof flags or counts are missing. They are useful for keeping cross-package
claim vocabulary aligned.

They do not invoke commands or inspect run directories. Calling code must
derive each field from actual validation, run, artifact, replay, parity, or
verification evidence. Hard-coded successful booleans prove only the report
validator.

## Contribution Rules

- Keep APIs deterministic and narrowly named by the contract they support.
- Keep filesystem work inside caller-provided or temporary directories.
- Never make a public runtime package require the testkit to build or run.
- Add helpers only when at least two suites share the same stable need.
- Return evidence-rich values; avoid helpers that hide why an assertion passed.
- Keep process-spawning helpers explicit because they are slower and less
  isolated than in-memory fixtures.
- Update the [Test Strategy](../quality/test-strategy.md) when a helper changes
  lane, fixture, snapshot, or evidence policy.

## Source Authorities

- `crates/bijux-dag-testkit/docs/CONTRACTS.md`
- `crates/bijux-dag-testkit/src/lib.rs`
- `crates/bijux-dag-testkit/src/workflows.rs`
- `crates/bijux-dag-testkit/src/fake_adapter.rs`
- `crates/bijux-dag-testkit/src/product_scenarios.rs`
- `evidence/dag/_meta/registries/evidence_registry.json`
