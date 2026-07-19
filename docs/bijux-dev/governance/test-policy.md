---
title: Test Policy
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Test Policy

Test policy exists to keep verification claims narrower than or equal to the
lane that produced them. It governs selection, expensive-test classification,
ignored coverage, retries, failure aggregation, and the evidence required to
call a result complete.

Command ownership and visible maintainer entrypoints are governed by
`contracts/foundation/maintainer_command_surface.v1.json`.

## Canonical Lanes

| Command | Required population | Execution policy | Honest result |
| --- | --- | --- | --- |
| focused `cargo nextest run` | explicit expression only | caller-selected profile and scope | named Rust behavior |
| focused `pytest` | explicit path, name, or marker expression | caller-selected coverage | named Python behavior |
| `make test` | fast Rust selection, then Python `not nightly` | Rust `ci` profile: retries disabled and no fail-fast; Python coverage threshold enforced | default cross-language test gate |
| `make test-rs` | fast Rust selection | local `default` profile, including its configured retry | local fast Rust lane, not the required release-profile lane |
| `make test-slow` | explicit `slow__` tests and exact roster entries | Rust only; ignored tests remain excluded | governed expensive Rust lane |
| `make test-all` | every Rust test, including ignored tests | `ci` profile, retries disabled, no selection filter, no fail-fast | complete Rust lane |
| `make test-nightly-py` | Python `nightly` marker | Python only, coverage disabled for the isolated marker lane | Python nightly lane |

`make test-slow` and `make test-all` do not include Python. Python nightly
tests are not aggregated into either command. Documentation, lint, audit,
packaging, and release validation remain separate gates.

## Fast And Slow Ownership

The fast and slow Rust lanes are complementary selections:

- explicit expensive tests use the `slow__` namespace;
- expensive tests with behavior-first names are listed exactly once in
  `configs/rust/nextest-slow-roster.txt`;
- the fast expression negates both sets;
- the slow expression selects both sets;
- the complete lane supplies no selection expression.

The roster must remain sorted, unique, and resolvable to real test functions.
Use it only when repeated measurement or an external-resource requirement
makes the test unsuitable for the default lane. Never roster a failing or
flaky test to obtain a green fast result.

Name a new inherently expensive test with `slow__` from the start. Do not use
the reserved prefix for ordinary behavior or duplicate the same test in the
roster.

## Ignored-Test Governance

Ignored tests are excluded from fast and slow lanes. The complete Rust lane
runs them with `--run-ignored all`.

- Flaky ignored tests are forbidden in release-facing DAG coverage.
- An ignored DAG test must belong to a governed nonstable portfolio with an
  explicit `experimental` or `internal` reason.
- Ignored Rust tests outside those portfolios are forbidden.
- Audits scan the complete DAG crate tree, including source-level unit-test
  helpers, not only top-level `tests/` directories.
- Refresh or maintenance work must use an explicit command such as
  `bijux-dev-cli docs write-dag-cli-reference`, not an ignored test used as a
  hidden task runner.

Run `bijux-dev-cli maintenance ignored-dag-tests` when ignored-test
classification or source-level DAG test helpers change.

## Retry And Failure Policy

The required Rust release profile and complete lane disable retries so a pass
cannot conceal an initial failure. Both set `fail-fast = false`, allowing
nextest to run the selected population and print a terminal summary even when
tests fail.

The local `default` profile permits one retry for development ergonomics.
Accordingly, `make test-rs` is useful feedback but is not interchangeable with
the Rust portion of `make test`.

Every retained result must preserve nextest's exit status after logging and
report its terminal summary. Passed, failed, slow, skipped, and leaky counts
remain evidence on unsuccessful runs. A wrapper must not short-circuit the
complete population, discard the summary, or turn `tee` success into gate
success.

## Frozen Commit Evidence

```bash
PINNED_REF=<commit> make test-all-frozen
```

This launches `make test-all` from a clean detached checkout under
`artifacts/<sha>/gates/test-all/frozen-repo/`. The launch message proves only
that execution started. The console log, final nextest report under
`artifacts/<sha>/gates/test-all/artifacts/`, and terminal status under
`artifacts/<sha>/background/` establish the result.

Do not report a frozen run as passed until the status file exists with success
and the final summary is present. A missing status means running, interrupted,
or orphaned.

## Required Claims

- A focused result names the exact test and does not stand in for a lane.
- A default `make test` result states whether slow Rust and Python nightly
  lanes were omitted.
- A complete Rust result does not imply Python, docs, lint, packaging, release,
  or external-platform validation.
- A release recommendation requires the release validation and compatibility
  evidence owned by the release process, not merely a green test lane.
- Advisory, simulated, skipped, filtered, and ignored selections remain
  visible in the evidence.

## What This Policy Protects

| Surface | Why the tests matter |
| --- | --- |
| fast review | expensive behavior cannot silently enter or disappear from the default lane |
| complete verification | failures do not prevent the remaining selected tests or terminal summary from running |
| release decisions | retries, omissions, and unsupported claims stay visible |
| ignored coverage | nonstable portfolios cannot masquerade as ordinary reliable tests |
| maintainers | command output cannot claim health beyond executable evidence |

## Implementation Anchors

- `crates/bijux-dev/tests/`
- `crates/bijux-dev/tests/ignored_test_hygiene_contracts.rs`
- `crates/bijux-dev/tests/shared_rust_make_contracts.rs`
- `crates/bijux-dev/src/suites/test.rs`
- `configs/rust/nextest-slow-roster.txt`
- `configs/rust/nextest.toml`
- `makes/rust.mk`

## Related Guidance

- [Quality Policy](quality-policy.md)
- [Repository Gates](../operations/repository-gates.md)
- [Evidence Collection](../operations/evidence-collection.md)
- [Testing and Validation](../../bijux-core/operations/testing-and-validation.md)
