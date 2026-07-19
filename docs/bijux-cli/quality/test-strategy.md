---
title: Test Strategy
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# Test Strategy

Use this page to choose the smallest test lane that proves a CLI change and to
understand what the repository-wide gates add. A green broad gate is useful,
but it is not a substitute for evidence in the suite that owns the contract.

## Proof Structure

The Rust CLI tests have three explicit entry points:

| Entry point | Owned proof |
| --- | --- |
| `tests/routing.rs` | parser normalization, aliases, route laws, registry behavior, help routing, and command-surface snapshots |
| `tests/integration.rs` | configuration, history, memory, plugins, resilience, REPL behavior, root commands, and complete process interactions |
| `tests/architecture.rs` | dependency boundaries, module ownership, and separation between routing, command, product, and plugin responsibilities |

Unit tests remain beside implementation code when the behavior is local to one
module. Workspace contract tests in `bijux-dev` defend cross-crate concerns
such as package boundaries, Make behavior, generated references, and release
evidence. Those tests complement the CLI suites; they do not replace them.

The Python package has its own tests under
`crates/bijux-cli-python/tests/python/`. They prove Python API behavior, native
extension loading, subprocess compatibility, and stable-release behavior at
the Python boundary. A Rust-only pass cannot establish that contract.

## Executable Lanes

| Command | Selection and purpose |
| --- | --- |
| `make test` | Rust fast tests plus Python tests marked `not nightly`; use for normal local feedback |
| `make test-slow` | Rust tests selected by the `slow__` namespace or the governed slow roster |
| `make test-all` | every Rust test, including ignored tests, with retries disabled |
| `make test-nightly-py` | Python tests carrying the `nightly` marker |
| `PINNED_REF=<ref> make test-all-frozen` | the complete Rust lane from an immutable checkout, launched in the background |

`make test-all` is deliberately a Rust completeness gate. It does not imply
that Python nightly compatibility has run. Release evidence that needs both
ecosystems must name both commands rather than treating one label as broader
than its implementation.

The nextest `ci` profile has `fail-fast = false`, so one failing Rust test does
not prevent the remaining selected tests from running. The complete lane also
uses `--run-ignored all` and `--retries 0`: ignored coverage is visible and an
unstable pass is not converted into success by retrying. The gate prints the
final nextest summary even when the command fails, then returns the original
failure status.

## Slow-Test Governance

Use `slow__` in the fully qualified Rust test name when slowness is an enduring
property of the scenario. The fast lane excludes that namespace and the slow
lane selects it.

Some expensive tests cannot be renamed without weakening an external contract.
Their exact fully qualified names belong in
`configs/rust/nextest-slow-roster.txt`. The roster is sorted, unique, and
validated against nextest discovery. Do not add a `slow__` test to the roster:
the namespace already governs it.

Ignored tests are not another spelling for slow tests. Ignoring a test means it
is outside normal execution for a specific reason, such as an experimental or
environment-dependent portfolio. The complete lane makes that decision
observable by running ignored tests explicitly.

## Durable Evidence

Tests should compare the strongest stable representation available:

- fixtures in `tests/data/fixtures/` provide controlled inputs
- golden CLI output in `tests/data/golden/cli_surface/` protects public text
  and structured envelopes
- routing snapshots in `tests/routing/snapshots/` expose parser and help drift
- minimized fuzz cases in `tests/fuzz/minimized_cases/` preserve regressions
  after nondeterministic discovery
- fuzz corpora and proptest regression files retain parser and routing inputs
  that previously reached an edge condition

Golden and snapshot changes are reviewable contract changes, not files to
refresh until a test passes. A change must be explained by the owning behavior
and inspected for accidental output drift.

Local Rust reports are written below
`artifacts/rust/test/<run-id>/`. The fast, slow, and complete logs are
`nextest.log`, `nextest-slow.log`, and `nextest-all.log`. Python test and
coverage output belongs below `artifacts/python/test/`. Frozen runs use
`artifacts/<sha>/`; their console log and exit status are under
`artifacts/<sha>/background/`.

## Change Rules

- Put a behavior test in the suite that owns the contract, then add broader
  coverage only when it proves a separate boundary.
- Test both the structured envelope and exit status for command failures.
- Keep architecture failures blocking; a behaviorally green command is not
  acceptable when it crosses an ownership boundary.
- Mark genuinely expensive Rust tests through the governed slow mechanisms
  rather than weakening assertions or hiding them with `ignore`.
- Run Python tests when native loading, subprocess behavior, packaging, or
  Python-facing compatibility changes.
- Preserve minimized failures and intentional output baselines in their
  governed locations.

## Authorities

- CLI integration, routing, and architecture suites:
  `crates/bijux-cli/tests/`
- Python package tests: `crates/bijux-cli-python/tests/python/`
- Rust lane adapter: `makes/bin/run_core_rust_gate.sh`
- Rust lane definitions: `makes/rust.mk`
- slow roster: `configs/rust/nextest-slow-roster.txt`
- nextest execution profiles: `configs/rust/nextest.toml`
- Python pytest and coverage configuration: `pyproject.toml`

## Continue Reading

- [Invariants](invariants.md)
- [Change Validation](change-validation.md)
- [Risk Register](risk-register.md)
