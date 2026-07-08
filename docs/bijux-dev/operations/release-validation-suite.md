---
title: Release Validation Suite
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-07
---

# Release Validation Suite

Use this page when a commit looks releaseable and you need proof that the
committed `HEAD`, not an ambient local workspace, can survive the full release
candidate gate.

The release validation suite is the hard boundary between "this change seems
ready" and "this commit is a publishable candidate." It proves formatting,
linting, testing, documentation, packaging, and dry-run publishing from a
clean release tree.

## Canonical Entrypoints

- local shell entrypoint: `make release-validate-rs`
- CI entrypoint: `make gh-release-validate`
- maintainer command surface: `cargo run -q -p bijux-dev --bin bijux-dev-cli -- release verify`

`make release-validate-rs` is the canonical local entrypoint. The CI wrapper
must stay thin and must delegate back to the same suite through
`make gh-release-validate`.

## Execution Model

The suite prepares a clean release tree from committed `HEAD` before any cargo
validation runs. It does not validate against ambient local edits. That keeps
local release evidence aligned with the publish surface used by CI and prevents
an uncommitted worktree from hiding a broken candidate.

The staged release tree patches the public DAG family back into the local
crates.io view for dry-run verification. That patch exists only to simulate the
topological release order for public crates that have not been published yet.
It must not be used to hide private crate dependencies inside public package
manifests.

The end-to-end release verification flow is:

```text
release.validation-suite -> release.readiness -> release.compatibility-matrix
```

The make target executes the cargo release checks. The maintainer command then
follows with the readiness report and compatibility matrix so the release lane
produces both validation status and the evidence consumed by release review.

## What This Suite Proves

| Proof point | Why it matters |
| --- | --- |
| clean-tree validation | release evidence must match committed content, not uncommitted local state |
| package listing and dry-run publish | crates.io-facing packaging boundaries stay honest before tagging |
| readiness and compatibility artifacts | release review gets concrete evidence, not verbal reassurance |

## Command Coverage

The suite must run the following commands exactly:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --all-features --no-deps
cargo package -p bijux-dag-core --list
cargo package -p bijux-dag-artifacts --list
cargo package -p bijux-dag-runtime --list
cargo package -p bijux-dag-app --list
cargo package -p bijux-dag-cli --list
cargo publish -p bijux-dag-core --dry-run --locked
cargo publish -p bijux-dag-artifacts --dry-run --locked
cargo publish -p bijux-dag-runtime --dry-run --locked
cargo publish -p bijux-dag-app --dry-run --locked
cargo publish -p bijux-dag-cli --dry-run --locked
cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture
```

The public DAG release family covered by this suite is:

- `bijux-dag-core`
- `bijux-dag-artifacts`
- `bijux-dag-runtime`
- `bijux-dag-app`
- `bijux-dag-cli`

The publish order remains governed by
`contracts/foundation/workspace_package_boundary.v1.json`.

## Outputs

The suite writes its release-candidate outputs under the repository
`artifacts/` tree:

- clean release tree: `artifacts/rust/release-validation/<run-id>/workspace/`
- shared target directory: `artifacts/rust/release-validation/<run-id>/target/`
- command logs and run outputs: `artifacts/rust/release-validation/<run-id>/`
- readiness report: `artifacts/release/readiness_report.json`
- compatibility matrix: `artifacts/release/compatibility_matrix.json`

Use the release-tree directory when a failure appears to depend on staged
content, packaging boundaries, or publish inputs. Use the readiness report and
compatibility matrix during release review and release-note preparation.

## How To Read A Failure

- formatter, clippy, test, doc, package, or publish failures belong to the release candidate; fix the candidate commit or its governed release inputs before tagging
- clean release-tree export failures belong to `.github/scripts/prepare_release_tree.py`; repair the export path so the candidate can be validated in isolation
- CI wrapper or workflow setup failures belong to `.github/workflows/release-validation.yml` and `makes/gh.mk`; repair the wrapper so hosted automation still runs the same suite as local maintainers

## When To Run It

Run the suite whenever a change is close enough to a release boundary that the
next question is release viability rather than implementation correctness. In
practice that means:

- before recommending a tag
- after changing public DAG crate packaging or publish metadata
- after changing release-tree preparation, release CI wiring, or release docs
- before relying on readiness or compatibility artifacts in release review

## Reader Shortcut

If a release command passes only in the live workspace and fails in the clean
release tree, the repository has not proved release readiness. The clean tree
is the truth surface.

## Related Surfaces

- [Release Operations](release-operations.md)
- [release-validation workflow](../gh-workflows/release-validation.md)
- [Release Surfaces](../makes/release-surfaces.md)
- [CI Targets](../makes/ci-targets.md)
