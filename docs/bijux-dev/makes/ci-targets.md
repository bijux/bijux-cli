---
title: CI Targets
audience: mixed
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# CI Targets

GitHub Actions jobs should delegate shell behavior to make targets so local and
hosted verification use the same commands, tool configuration, artifact
locations, and exit semantics. Workflow YAML selects a gate; the Make layer
owns how that gate runs.

## CI-Aligned Targets

| Target | Contract |
| --- | --- |
| `make gh-fmt` | install the repository toolchain, then verify Rust and Python formatting without modifying files |
| `make gh-lint` | run Clippy with warnings denied and the Python lint checks |
| `make gh-security` | run the repository security aggregate |
| `make gh-test` | install test prerequisites, then delegate to `make test` |
| `make gh-release-validate` | run the canonical Rust release-candidate suite from a clean tree prepared from committed `HEAD` |
| `make gh-docs-install` | install and report the documentation toolchain used by hosted docs jobs |
| `make gh-release-wait-for-ci` | wait for the governed required checks before release work proceeds |

Do not copy a target recipe into a workflow. A local and hosted command with
the same label but different shell steps is not one gate.

## Test Lane Mapping

The root labels are intentionally narrower than their names may suggest:

| Target | What runs | What does not run |
| --- | --- | --- |
| `make test` | `make test-release-rs`, then Python tests marked `not nightly` | Rust slow and ignored tests; Python nightly tests |
| `make test-rs` | non-ignored Rust tests outside the `slow__` namespace and governed slow roster | Python; Rust slow and ignored tests |
| `make test-slow` | the Rust `slow__` namespace and governed slow roster | Python; unrelated ignored tests |
| `make test-all` | every Rust test, including ignored tests, with retries disabled | all Python tests |
| `make test-nightly-py` | Python tests marked `nightly` | Rust and normal Python tests |

`make test-release-rs` uses the nextest `ci` profile for the fast selection.
That profile disables retries, uses one test thread, and has
`fail-fast = false`. The plain `make test-rs` and `make test-slow-rs` targets
use their configured local profile, which may retry according to
`configs/rust/nextest.toml`.

`make test-all-rs` supplies no selection expression, adds
`--run-ignored all --retries 0`, and uses the `ci` profile. This is the Rust
completeness lane, not a repository-wide claim about Python. Evidence that
needs every Python class must run the normal and nightly Python targets
explicitly.

A multi-prerequisite Make target stops according to Make failure semantics. If
complete evidence from independent ecosystems is required after one has
failed, run and record those targets independently rather than assuming a later
prerequisite executed.

Use [Release Validation Suite](../operations/release-validation-suite.md) for
the exact release-candidate command inventory and artifact outputs behind
`make gh-release-validate`.

## Rust Failure Semantics

Rust nextest profiles used by the required and complete gates set
`fail-fast = false`. All selected Rust tests therefore continue after an
individual failure. The gate captures the full console stream, extracts the
last nextest `Summary [...]` line, prints it as `nextest-summary:`, and then
returns the original nextest status.

The summary remains diagnostic evidence, not a replacement status. A run with
failed or leaky tests remains failed even though the summary and report were
successfully written.

Rust reports use these paths:

| Lane | Report |
| --- | --- |
| fast | `artifacts/rust/test/<run-id>/nextest.log` |
| slow | `artifacts/rust/test/<run-id>/nextest-slow.log` |
| complete | `artifacts/rust/test/<run-id>/nextest-all.log` |

The Core adapter builds `bijux-dev-cli` and `bijux-dag` in the isolated Rust
target directory before these lanes because tests invoke those binaries.

## Frozen Commit Gates

Use a frozen gate when evidence must be attributable to one immutable commit:

```bash
PINNED_REF=<ref> make test-all-frozen
PINNED_REF=<ref> make lint-frozen
PINNED_REF=<ref> make audit-frozen
```

The launcher resolves `<ref>` to a commit, creates or verifies a detached clean
checkout at `artifacts/<sha>/frozen-repo/`, and starts the selected Make target
in a detached background process. Returning from the launcher means the
process started; it does not mean the gate passed.

Inspect the run through:

- `artifacts/<sha>/background/<gate>.console.log` for complete output
- `artifacts/<sha>/background/<gate>.pid` for the launched process
- `artifacts/<sha>/background/<gate>.exit.status` for the final numeric status
- `artifacts/<sha>/background/<gate>.meta` for the resolved commit and paths
- `artifacts/<sha>/rust/` for Rust reports, Cargo state, and test products

The status file is removed before launch and written only after the gate
finishes. Its absence means the run has not published a final result. For a
frozen complete test, the console log still contains the final
`nextest-summary:` line on test failure because the pinned checkout runs the
same complete Rust gate.

`PINNED_REF` is the canonical selector. The launcher accepts
`TEST_ALL_FROZEN_REF` for compatibility with existing invocations, but new
automation and documentation should use `PINNED_REF`.

## Source Authorities

- hosted target composition: `makes/gh.mk`
- Core-specific Rust preparation: `makes/bin/run_core_rust_gate.sh`
- repository Rust profiles and release suite: `makes/rust.mk`
- Python test markers and reports: `makes/python.mk` and `pyproject.toml`
- shared Rust lane implementation:
  `.bijux/shared/bijux-makes-rs/scripts/rust_gate.sh`
- shared frozen launcher:
  `.bijux/shared/bijux-makes/scripts/run_pinned_gate.sh`
- nextest policy: `configs/rust/nextest.toml`
- slow-test roster: `configs/rust/nextest-slow-roster.txt`

The files below `.bijux/shared/` are synchronized standards. Fix shared
behavior in `bijux-std` and refresh the managed content; do not hand-edit a
downstream copy.

## Next Reads

- [Release Validation Suite](../operations/release-validation-suite.md)
- [Release Surfaces](release-surfaces.md)
- [gh-workflows](../gh-workflows/index.md)
- [CI and Automation](../operations/ci-and-automation.md)
