---
title: Test Strategy
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Test Strategy

DAG tests must prove the boundary that owns a claim. A command-level success
test cannot replace graph-kernel properties, an in-memory artifact assertion
cannot replace retained filesystem verification, and a fake adapter cannot
replace a real process or container boundary.

## Coverage By Owner

| Owner | Required proof |
| --- | --- |
| `bijux-dag-core` | strict parsing, diagnostics, canonicalization, topology, graph identity, lowering |
| `bijux-dag-artifacts` | serialized models, schema compatibility, paths, atomic writes, hashes, lineage, finalization |
| `bijux-dag-runtime` | scheduling, adapters, policy, retries, timeout, cache, replay, retained execution evidence |
| `bijux-dag-app` | route classification, typed responses, output modes, inspection, comparison, recovery |
| `bijux-dag-cli` | executable startup, help, completion, exit mapping, end-to-end smoke |
| `bijux-dev` | cross-package architecture, evidence, release, and governance contracts |

Cross-package tests belong at the narrowest layer that can observe the complete
claim. Keep pure graph semantics out of CLI snapshots and production behavior
out of testkit helpers.

## Test Lanes

| Command | Intended use | Selection |
| --- | --- | --- |
| `make test-rs` | local fast Rust feedback | non-ignored tests excluding `slow__` names and roster entries |
| `make test-release-rs` | required stable Rust release behavior | fast selection under the CI nextest profile |
| `make test-slow` | expensive governed Rust coverage | `slow__` names plus slow-roster entries |
| `make test-all` | complete Rust verification | all Rust tests with ignored portfolios included |
| `TEST_ALL_FROZEN_REF=<commit> make test-all-frozen` | immutable full verification | `test-all` from a pinned committed checkout |

The repository root `make test` combines the required Rust release lane with
the Python suites. DAG-specific Rust development can use the narrower targets
above.

Tests that repeatedly exceed the fast-lane budget use a `slow__` namespace.
Existing expensive tests that cannot yet be renamed are listed by exact test
name in `configs/rust/nextest-slow-roster.txt`. The roster is an explicit
selection contract, not a place to hide unstable tests.

## Ignored Tests

Ignored Rust tests are permitted only for experimental or internal DAG command
portfolios recorded in
`configs/dag/policy/release_test_lane_governance.json`. Each record identifies
the test, owning surface class, rationale, and full-lane command.

Stable behavior may not depend on ignored coverage. A flaky test is not a
valid ignored portfolio. The hygiene contract in
`crates/bijux-dev/tests/ignored_test_hygiene_contracts.rs` rejects ignored
tests that are absent from governance or carry another reason.

`make test-all` must execute the governed ignored portfolios and continue
through the full nextest run so its final summary reports passed, failed, slow,
and skipped counts. A targeted test command is useful during repair but does
not replace that complete evidence.

## Fixture Authority

Use the fixture form that matches the claim:

- inline values for one local unit invariant;
- crate-owned fixtures for package serialization and compatibility;
- `evidence/dag/authoring/` for governed authoring examples, patterns, and
  rejection cases;
- registry-owned evidence for cross-package claims that need stable asset
  identity and consumer tracking;
- retained run snapshots for filesystem shape, trace, index, and payload
  assertions.

Prefer evidence-registry identifiers over copied relative paths for
cross-package consumers. A registry change must preserve ownership and update
every declared consumer. Compatibility path remapping in `bijux-dag-testkit`
exists for existing suites; new tests should use canonical
`evidence/dag/...` paths.

Do not regenerate a snapshot merely because it changed. First classify the
change as intended contract evolution, nondeterministic noise, or regression.
Snapshot normalization may remove timestamps, process IDs, and captured build
identity. It must not erase statuses, digests, node identities, failure codes,
paths, or other fields under test.

## Testkit Role

`bijux-dag-testkit` centralizes deterministic graph builders, evidence readers,
run snapshots, fake-adapter scenarios, and shared assertions. It reduces
fixture drift; it does not own product semantics.

The package is `publish = false`. Production crates must not depend on it at
runtime, and public packages must build and package without it. Shared helpers
should expose the evidence being asserted rather than collapsing a complex
behavior into one unexplained boolean.

Fake-adapter scenarios cover success, failure, timeout classification, missing
output, corrupt output, large output, and harness error paths. They do not
exercise operating-system process control, container engines, network
behavior, real clocks, child-process termination, or external credentials.
Those claims require tests at the runtime or executable boundary.

Product scenario report builders reject incomplete reports supplied by tests.
They do not run validation, planning, execution, replay, or verification
themselves. A scenario report is useful only when the calling test derives its
fields from real evidence.

## Required Change Evidence

| Change | Minimum targeted evidence |
| --- | --- |
| graph field or diagnostic | positive, negative, round-trip, and identity tests |
| retained record or schema | model round-trip, schema fixture, old/new compatibility, corruption refusal |
| adapter or backend | conformance, real boundary, failure classification, retained trace |
| cache key or proof | hit, miss reason, tamper refusal, lineage change |
| replay behavior | source verification, focused closure, mismatch reason, missing evidence refusal |
| stable command or envelope | parser, route, text/JSON parity, exit code, executable smoke |
| experimental/internal route | governed ignored portfolio plus full-lane coverage |

When a fix changes expected behavior, update the implementation, contract,
fixtures, tests, and public explanation in the same review. Do not weaken an
assertion to accommodate unexplained output.

## Review Standard

A DAG test change is ready when:

- the test runs in the correct lane and ownership layer;
- deterministic inputs and artifact roots are explicit;
- assertions cover the actual contract rather than only command success;
- fake or normalized evidence does not conceal the relevant boundary;
- failure and refusal behavior is tested where trust depends on it;
- ignored and slow classification follows governed policy;
- the narrow test passes, and the next complete lane is identified.

Use [Change Validation](change-validation.md) for repository review gates and
[Artifact Contracts](../interfaces/artifact-contracts.md) for retained-evidence
assertions.
