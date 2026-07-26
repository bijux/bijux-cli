---
title: Release Validation Workflow
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-27
---

# Release Validation Workflow

`.github/workflows/release-validation.yml` runs the canonical Rust release
candidate suite against the checked-out commit. It is a hosted adapter around
`make gh-release-validate`, not a separate release definition.

## Events And Cancellation

The workflow runs for:

- pull requests opened by repository contributors;
- deliberate manual dispatches when maintainers need fresh release evidence.

It does not run on pushes to `main`, because the pull request already proves
the merge candidate and repeating the same heavy suite after merge consumes
runner quota without adding evidence. Dependabot pull requests skip the job and
remain dependency notifications until a maintainer deliberately incorporates
the update into a contributor branch.

Concurrency is scoped by workflow and Git ref. A newer run cancels an older
in-progress run for the same ref. Cancellation means superseded, not failed or
passed; the incomplete run cannot support a release decision.

The workflow has read-only repository contents permission, a 60-minute job
timeout, and one Ubuntu 24.04 job.

## Hosted Environment

The job checks out full history, installs Python 3.11, selects Rust `1.86.0`
with `rustfmt` and Clippy, and restores the shared Rust cache. Full Git history
is required because the suite exports committed `HEAD` and records source
identity. A restored cache can reduce runtime but is not release evidence.

The toolchain in this workflow matches `rust-toolchain.toml` and the workspace
MSRV. The synchronized generic CI and release environment currently contain a
separate `1.85.0` mismatch; a green release-validation run does not conceal or
repair that standards drift.

## Delegation

The only repository command run by the workflow is:

```bash
make gh-release-validate
```

`makes/gh.mk` delegates that target to `make release-validate-rs`. The Rust
fragment:

1. exports committed `HEAD` to an isolated release tree;
2. stamps the workspace release version;
3. patches the staged public DAG crates into the isolated crates.io view;
4. runs formatting, Clippy, workspace tests, Rust documentation, package file
   listings, locked publish dry-runs, and the installed CLI smoke test.

The suite also bounds Cargo parallelism inside that clean-tree run. This is not
a workflow-only shortcut; it is part of the repository-owned release suite so
hosted execution does not exhaust runner disk during parallel link phases while
local and CI behavior remain aligned.

The suite is deliberately sequential. It stops when a required command fails,
so later command logs may be absent. That is fail-fast release validation, not
a complete inventory of every possible candidate defect.

The exact command list, package order, artifact paths, and proof limits live in
[Release Validation Suite](../operations/release-validation-suite.md) and
`configs/dag/release/release_validation_suite.json`.

## Evidence Retention

Local execution retains the prepared tree, target directory, and per-command
logs under:

```text
artifacts/rust/release-validation/<run-id>/
```

The hosted workflow currently does not upload that directory with an artifact
action. GitHub therefore retains the workflow and step logs, but the isolated
tree and report files disappear with the runner.

Do not cite a downloadable release-validation artifact for this workflow
unless the workflow is changed to upload one. For durable candidate evidence,
run the local suite against the immutable commit or add an explicit,
checksum-preserving upload step through the owning workflow policy.

## Reading A Result

| Result | Meaning |
| --- | --- |
| success | every required release-suite command completed for the checked-out commit |
| failure in a Cargo or package command | the candidate or governed release input failed at that boundary |
| release-tree preparation failure | committed source could not be exported or stamped correctly |
| setup or cache failure | hosted workflow infrastructure failed before candidate proof |
| cancelled | a newer run superseded this run; no pass claim |
| timed out | the suite did not complete within 60 minutes; no pass claim |

A green result covers the Rust release suite only. It does not include Python
tests or publication, the MkDocs site, dependency security audits, benchmarks,
soak tests, live platforms, or actual registry uploads.

## Diagnose Drift

If local and hosted results disagree:

1. compare exact source SHA;
2. compare Rust, Python, Cargo tool, and runner versions;
3. compare the clean-tree artifact with the checked-out commit;
4. identify the first different command or environment input;
5. repair `makes/` when delegation is wrong, or the workflow owner when hosted
   setup is wrong.

Do not add workflow-only exclusions to make hosted validation green.

## Related Guidance

- [Release Validation Suite](../operations/release-validation-suite.md)
- [Repository CI](ci.md)
- [Release Operations](../operations/release-operations.md)
- [Make CI Targets](../makes/ci-targets.md)
