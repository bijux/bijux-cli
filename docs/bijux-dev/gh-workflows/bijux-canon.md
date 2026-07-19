---
title: Bijux Canon
audience: maintainer
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Bijux Canon

`bijux-canon.yml` is the repository's blocking governance workflow for DAG
packages and their maintainer control plane. It checks source policy,
cross-platform governance behavior, generated evidence, package readiness, and
public API drift. It is not the complete `bijux-core` test suite and must not be
reported as one.

The workflow is the authority for hosted execution. This page explains its
contract; when prose and workflow differ, treat the workflow as executable
truth and correct the documentation in the same change.

## When It Runs

Maintainers can start the workflow manually. Pull requests and pushes to
`main` also start it when a governed input changes:

- workspace manifests, the lockfile, or the Rust toolchain pin;
- DAG and `bijux-dev` crates;
- DAG and Rust configuration;
- Make implementation, DAG documentation, or maintainer documentation;
- evidence assets, GitHub scripts, or workflow definitions.

Path filtering deliberately avoids running this matrix for unrelated package
or documentation changes. A change outside those paths receives no Canon
evidence. Reviewers must not infer a skipped workflow result from an older run.

Concurrency is keyed by workflow and Git ref. A newer run cancels an older run
for the same ref so that the visible result belongs to the latest commit.

## Execution Contract

Every matrix lane:

- checks out the same commit;
- uses Rust `1.86.0`;
- restores the shared Rust build cache;
- has a lane-specific timeout;
- is blocking at the GitHub job level.

The matrix uses `fail-fast: false`. One failed lane therefore does not cancel
the other lanes, and the completed run exposes the full set of independent
outcomes. Commands within one lane still use normal shell failure behavior:
the first failing command ends that lane.

No suite command runs in advisory mode. A selected policy, test, contract, or
documentation failure produces a failed job. The Rust quality selection
explicitly includes its slow suite because that suite owns the Clippy check.

## Matrix

| Lane | Required proof |
| --- | --- |
| Rust Format | `cargo fmt --all -- --check` leaves the checkout unchanged. |
| Rust Lint | Dependency policy, Clippy with warnings denied, formatting, and supply-chain checks pass. |
| Rust Test on Ubuntu | The governance test domain passes on Ubuntu 24.04. |
| Rust Test on macOS | The same governance test domain passes on macOS 14. |
| Compatibility Fixture Drift | The compatibility contract suite passes against checked-in fixtures. |
| Dependency Audit | `cargo-audit` `0.22.1` and the repository security command accept the dependency state. |
| Docs Build | Rust API documentation builds and the maintainer documentation suite passes. |
| Ecosystem Contracts | Control-plane and evidence-suite contract tests pass. |
| Evidence Verify | Release, battle, cache, replay, and consumer evidence verifies; governed reports regenerate without drift. |
| Package Dry Run | Package-boundary contracts pass and release packages survive Cargo publish dry runs. |
| Public API Drift | The pinned nightly toolchain and `cargo-public-api` accept the governed public surface. |
| Repo Health | Hotspot and planner reports regenerate without drift and file-size guardrails pass. |
| Schema Governance | The schema changelog regenerates without drift and schema contracts pass. |

The lane names are review labels, not substitutes for their commands. For
example, Docs Build proves Rust API documentation and the `bijux-dev-dag`
documentation suite; it does not build the MkDocs site.

## Evidence

Lanes that generate reports upload their declared path even after a failed run.
This preserves diagnostic output from checks, cross-platform tests,
compatibility, dependency review, documentation, evidence verification,
package dry runs, public API checks, and schema governance.

Artifact upload uses `if-no-files-found: warn`. A green lane with an expected
report missing is therefore incomplete evidence even though GitHub accepted
the upload step. Review the artifact inventory as well as the job result when
the report itself supports a release or governance claim.

Several lanes prove checked-in state through `git diff --exit-code`. A failure
means the generator and repository disagree. Review and commit a semantically
correct generated result; do not bypass the comparison or hand-edit a
generated report to match an expected diff.

## Release Proof

The Release Proof job runs after every matrix lane succeeds, but only for a
manual dispatch or a push to `main`. It does not run for pull requests. The job
verifies the release evidence set, generates the release evidence report, and
uploads `artifacts/reports`.

This job demonstrates that the accepted commit can produce the governed
release-evidence bundle. It does not publish a crate, create a GitHub release,
or establish production readiness beyond the assets registered in that
evidence set.

## Reproducing a Failure

Use the exact command shown in the failed workflow lane from the repository
root. Preserve `--locked`, pinned tool versions, selected domains, and operating
system when those details affect the result. There is no single Make target
that reproduces the complete cross-platform matrix.

Record:

- the commit SHA and lane name;
- the exact command and runner platform;
- the first causal failure, not only the final nonzero status;
- generated report differences and uploaded artifact names;
- whether local reproduction used the same toolchain and lockfile.

Do not replace a blocking invocation with `--advisory`, remove
`--include-slow`, narrow a domain, or accept missing artifacts to obtain a
green result.

## What Green Does Not Prove

A successful Canon run does not prove:

- the complete Rust or Python test suites;
- ignored, nightly, benchmark, fuzzing, or live-platform behavior;
- the MkDocs public site;
- registry publication or installation from a registry;
- performance, scalability, or production deployment readiness.

Use the repository-wide test, documentation, release-validation, and
deployment gates for those claims.

## Next Reads

- [CI workflow](ci.md)
- [Release validation](release-validation.md)
- [Documentation deployment](deploy-docs.md)
- [Evidence collection](../operations/evidence-collection.md)
