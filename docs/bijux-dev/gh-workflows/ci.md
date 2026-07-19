---
title: Repository CI
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# Repository CI

`.github/workflows/ci.yml` is the required pull-request and merge-queue
validation workflow. It is synchronized from `bijux-std`; this repository owns
the delegated make targets, not the generated workflow structure.

## Triggers And Concurrency

- `pull_request` targeting `main`;
- `merge_group` targeting `main`;
- one active run per workflow and ref, with an older in-progress run cancelled
  when a newer commit supersedes it.

The workflow does not run on a direct `push` event. Branch protection and merge
policy must therefore require the pull-request or merge-group checks rather
than assume a post-push CI run exists.

Dependabot pull requests conditionally skip the four repository jobs in this
workflow. Dependency policy and any separate Dependabot validation must not be
misreported as execution of these skipped jobs.

## Job Contract

| Job | Hosted command | Repository authority |
| --- | --- | --- |
| Formatting | `make gh-fmt` | Rust and Python format checks without source mutation |
| Lint | `make gh-lint` | Rust and Python lint policy |
| Security | install pinned security tools, then `make gh-security` | dependency and repository security gates |
| Tests | install pinned nextest, then `make gh-test` | required Rust release lane and Python tests |

`make gh-test` delegates to the root `make test` contract. It is not the
complete ignored/slow Rust portfolio; use the governed full and frozen lanes
when that evidence is required.

Release packaging, publish dry runs, documentation, and smoke validation belong
to `.github/workflows/release-validation.yml` through
`make gh-release-validate`. A green repository CI workflow alone is not a
release-readiness claim.

## Environment Authority

The generated workflow selects the runner, Python version, Rust toolchain,
components, caches, and tool installation. Those values must agree with:

- `Cargo.toml` workspace `rust-version`;
- `rust-toolchain.toml`;
- `configs/rust/clippy.toml` MSRV;
- Python requirements in `crates/bijux-cli-python/pyproject.toml`;
- pinned tool compatibility in `makes/gh.mk`.

A mismatch is standards drift, not a documentation choice. Correct the owning
configuration in `bijux-std`, refresh synchronized content, and validate the
shared checksum. Do not patch this generated workflow locally or change the
handbook to conceal the mismatch.

## Failure Routing

1. Record the workflow run, source commit, event, job, and final status.
2. Run the delegated make target from the same commit.
3. Compare toolchain, installed tools, environment, and selected test lane.
4. Fix repository target behavior locally when local and hosted commands share
   the same bad behavior.
5. Fix runner setup or generated workflow behavior in `bijux-std`, then refresh
   this repository.

Do not add workflow-only shell logic to make a red local target appear green in
GitHub Actions.

## Related Operations

- [CI and Automation](../operations/ci-and-automation.md)
- [CI Targets](../makes/ci-targets.md)
- [Repository Gates](../operations/repository-gates.md)
- [Release Validation](release-validation.md)
- [Documentation Deployment](deploy-docs.md)
