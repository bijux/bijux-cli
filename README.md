# bijux-core

<!-- bijux-core-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-core/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-core/workflows/release-crates/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml)
[![PyPI Publish](https://github.com/bijux/bijux-core/workflows/release-pypi/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-core?display_name=tag&label=release)](https://github.com/bijux/bijux-core/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-2%20packages-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-core)
[![Published packages](https://img.shields.io/badge/published%20packages-2-2563EB)](https://github.com/bijux/bijux-core/tree/main/crates)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-dag docs](https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-dag/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-core` is the shared workspace for two public Bijux products:

- `bijux`, the root command runtime for apps, plugins, configuration,
  diagnostics, and interactive workflows
- `bijux-dag`, the local-first DAG system for validation, planning, execution,
  replay, artifact inspection, and evidence-backed comparison

The repository also carries the internal crates that keep those products
packaged, tested, documented, and released from one reviewable tree.

## What Ships Today

| Surface | Delivery form | What it is for |
| --- | --- | --- |
| `bijux` | Rust crate, Python distribution, release bundles | operator-facing command runtime with apps, plugins, config, history, memory, REPL, and diagnostics |
| `bijux-dag` | Rust crates and release bundles | local DAG validation, planning, execution, replay, diff, artifact inspection, and verification |
| `bijux-dev` | repository-internal crate and binaries | maintainer diagnostics, contracts, inventories, and release proof |

The current workspace release line is `0.4.0`.

## Stable Product Boundary

`v0.4.0` ships a usable local product surface today, but not every repository
route is a public promise.

- `bijux` supports the visible root command surface shown by `bijux --help`,
  including runtime health, app and plugin routing, layered config, history,
  memory, and REPL workflows.
- `bijux-dag` supports the visible `bijux-dag --help` surface for local DAG
  work: `validate`, `plan`, `run`, `replay`, `runs`, `artifact`,
  `artifact-inspect`, `diff`, `explain`, `verify`, `doctor`, `cache`,
  `version`, `commands`, and `completions`.
- branch-backed DAG workflows are part of that stable local operator surface;
  retained runs record the selected branch decision, the skipped lane, and the
  join-node trigger outcome.
- local container-backed DAG nodes are part of that stable operator surface
  when a supported engine such as Docker is available on `PATH`; the runtime
  records engine and image identity and fails clearly when the engine is
  unavailable.
- Experimental DAG routes remain callable by explicit path, but they are not
  part of the stable compatibility lane.
- Simulated and maintainer-only DAG namespaces require
  `BIJUX_DAG_ENABLE_SIMULATED=1` or `BIJUX_DAG_ENABLE_INTERNAL=1`.
- The current internal schedule lane is repository-tested for cron preview,
  durable submission, queue dispatch, and queue-to-run linkage, but it remains
  outside the stable `v0.4.0` public operator contract.
- Cluster-backed Kubernetes or HPC execution, public remote workers, and
  public enterprise or federation APIs are not part of the `v0.4.0` public
  product boundary.

## Package Families

<!-- bijux-core-package-map:generated:start -->
The public package families in this repository are:

| Package | Purpose | Links |
| --- | --- | --- |
| `bijux-cli` | Public release family for the `bijux` command runtime, spanning the Rust crate, Python distribution, and release bundle. | <a href="https://crates.io/crates/bijux-cli"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-cli?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-cli"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://pypi.org/project/bijux-cli/"><img alt="PyPI" src="https://img.shields.io/pypi/v/bijux-cli?label=PyPI&logo=pypi" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/"><img alt="Docs" src="https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli"><img alt="GHCR" src="https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
| `bijux-dag` | Public DAG release family for deterministic graph authoring, artifact identity, execution, and the stamped `bijux-dag` command bundle. | <a href="https://crates.io/crates/bijux-dag-cli"><img alt="Crates.io" src="https://img.shields.io/crates/v/bijux-dag-cli?label=crates.io&logo=rust" height="18"></a> <a href="https://docs.rs/bijux-dag-cli"><img alt="Rust docs" src="https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white" height="18"></a> <a href="https://bijux.io/bijux-core/bijux-dag/"><img alt="Docs" src="https://img.shields.io/badge/docs-bijux--dag-2563EB?logo=materialformkdocs&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag"><img alt="GHCR" src="https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github&logoColor=white" height="18"></a> <a href="https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-cli"><img alt="Source" src="https://img.shields.io/badge/source-181717?logo=github&logoColor=white" height="18"></a> |
<!-- bijux-core-package-map:generated:end -->

Within those families, the workspace currently contains:

- public `bijux` crates: `bijux-cli`
- public `bijux-dag` crates: `bijux-dag-core`, `bijux-dag-artifacts`,
  `bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-cli`
- repository-internal support crates: `bijux-cli-python`, `bijux-dag-testkit`,
  `bijux-dev`

The canonical publication boundary lives in
[`docs/bijux-core/foundation/package-boundary.md`](docs/bijux-core/foundation/package-boundary.md)
and `contracts/foundation/workspace_package_boundary.v1.json`.

## Repository Layout

- [`crates/bijux-cli`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli): Rust runtime crate behind the `bijux` executable.
- [`crates/bijux-cli-python`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-cli-python): Python bridge package and native extension surface for CLI runtime distribution.
- [`crates/bijux-dag-core`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-core): DAG schema, invariants, canonicalization, hashing, and replay/diff primitives.
- [`crates/bijux-dag-runtime`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-runtime): DAG execution engine and run lifecycle behavior.
- [`crates/bijux-dag-app`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-app): DAG command orchestration, response modeling, and render flows.
- [`crates/bijux-dag-cli`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-cli): thin binary entrypoint for `bijux-dag`.
- [`crates/bijux-dag-artifacts`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-artifacts): artifact and persistence utilities for DAG evidence handling.
- [`crates/bijux-dag-testkit`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dag-testkit): repository-internal fixtures and helpers for DAG contract testing.
- [`crates/bijux-dev`](https://github.com/bijux/bijux-core/tree/main/crates/bijux-dev): maintainer control plane for governance, diagnostics, release contracts, and evidence tooling.
- [`docs/`](https://github.com/bijux/bijux-core/tree/main/docs): canonical handbook set for repository, CLI, DAG, and maintainer surfaces.
- [`makes/`](https://github.com/bijux/bijux-core/tree/main/makes): make modules for root workflows, Rust/Python validation, DAG commands, docs, and release automation.

## Workspace Pillars

- [`crates/`](https://github.com/bijux/bijux-core/tree/main/crates) contains
  the Rust package boundaries for public products and internal support crates.
- [`docs/`](https://github.com/bijux/bijux-core/tree/main/docs) contains the
  published handbooks for repository, CLI, DAG, and maintainer surfaces.
- [`contracts/`](https://github.com/bijux/bijux-core/tree/main/contracts)
  contains machine-checkable compatibility, schema, and release-boundary
  contracts.
- [`makes/`](https://github.com/bijux/bijux-core/tree/main/makes) contains the
  repository make modules used by local workflows and CI.
- [`evidence/dag/`](https://github.com/bijux/bijux-core/tree/main/evidence/dag)
  contains governed DAG fixtures, scenarios, and proof material used across
  docs, tests, and release verification.

## Quick Start

Local builds, CI, and release jobs all use the pinned Rust `1.86.0` toolchain
declared in `rust-toolchain.toml`.

Install the public command surfaces:

```bash
cargo install bijux-cli
cargo install bijux-dag-cli
python -m pip install bijux-cli
```

Build and test from repository root:

```bash
cargo check --workspace
cargo test --workspace
```

Inspect product command surfaces:

```bash
cargo run -p bijux-cli --bin bijux -- --help
cargo run -p bijux-dag-cli --bin bijux-dag -- --help
cargo run -p bijux-dag-cli --bin bijux-dag -- validate --help
cargo run -p bijux-dag-cli --bin bijux-dag -- commands --all
```

For the repository-backed internal schedule workflow that proves cron preview,
same-slot suppression, queue dispatch, and explicit run linkage without
claiming a public scheduler service, use
[`docs/bijux-dag/operations/guides/scheduled-catalog-refresh-workflow.md`](docs/bijux-dag/operations/guides/scheduled-catalog-refresh-workflow.md).

Run a real DAG workflow against the repository file-processing example:

```bash
SOURCE_DIR="$(pwd)/evidence/dag/authoring/examples/file-processing-source"

cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/dag/authoring/examples/file-processing-report.dag.json

cargo run -p bijux-dag-cli --bin bijux-dag -- run \
  evidence/dag/authoring/examples/file-processing-report.dag.json \
  --out artifacts/file-processing-runs \
  --run-id file-processing-source \
  --cache readwrite \
  --cache-dir artifacts/file-processing-cache \
  --input "source_dir=${SOURCE_DIR}" \
  --input "report_title=Repository File Processing Report"
```

Inspect the retained report artifact, lineage, focused replay boundary, and
promotion path:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- artifact-inspect \
  artifacts/file-processing-runs/run-file-processing-source \
  render_report:report.md

cargo run -p bijux-dag-cli --bin bijux-dag -- artifact lineage \
  artifacts/file-processing-runs/run-file-processing-source \
  --json

cargo run -p bijux-dag-cli --bin bijux-dag -- replay --json \
  --source-run-id file-processing-source \
  --source-run-root artifacts/file-processing-runs \
  --out artifacts/file-processing-runs \
  --run-id file-processing-rerun \
  --from-node render_report

cargo run -p bijux-dag-cli --bin bijux-dag -- artifact promote \
  artifacts/file-processing-runs/run-file-processing-source \
  render_report:report.md \
  --deliverables-root artifacts/file-processing-deliverables \
  --to release \
  --json
```

Run a structured data pipeline against the repository regional sales example:

```bash
ORDERS_CSV="$(pwd)/evidence/dag/authoring/examples/regional-sales-source/orders.csv"
TARGETS_JSON="$(pwd)/evidence/dag/authoring/examples/regional-sales-source/targets.json"

cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/dag/authoring/examples/regional-sales-pipeline.dag.json

cargo run -p bijux-dag-cli --bin bijux-dag -- run \
  evidence/dag/authoring/examples/regional-sales-pipeline.dag.json \
  --out artifacts/regional-sales-runs \
  --run-id regional-sales-cold \
  --cache readwrite \
  --cache-dir artifacts/regional-sales-cache \
  --input "orders_csv=${ORDERS_CSV}" \
  --input "targets_json=${TARGETS_JSON}" \
  --input "report_title=Regional Revenue Attainment"
```

Run a real container-backed packaging workflow against the repository release
note example:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/release-note-source/weekly-update.md"

cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/dag/authoring/examples/release-note-bundle.dag.json

cargo run -p bijux-dag-cli --bin bijux-dag -- run --json \
  evidence/dag/authoring/examples/release-note-bundle.dag.json \
  --out artifacts/release-note-bundle-runs \
  --run-id release-note-bundle \
  --input "source_note=${SOURCE_NOTE}" \
  --input "bundle_label=Release Brief"
```

Inspect the retained container trace and outputs:

```bash
cat artifacts/release-note-bundle-runs/run-release-note-bundle/nodes/package_bundle/trace.json
cat artifacts/release-note-bundle-runs/run-release-note-bundle/nodes/package_bundle/outputs/bundle/release-note.txt
```

Run a real branch-backed publishing workflow against the repository audience
bulletin example:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/audience-branch-source/team-update.md"

cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/dag/authoring/examples/audience-branch-bulletin.dag.json

cargo run -p bijux-dag-cli --bin bijux-dag -- run --json \
  evidence/dag/authoring/examples/audience-branch-bulletin.dag.json \
  --out artifacts/audience-branch-runs \
  --run-id audience-branch-technical \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=technical"
```

Inspect the retained branch and join evidence:

```bash
cat artifacts/audience-branch-runs/run-audience-branch-technical/nodes/choose_audience_lane/trace.json
cat artifacts/audience-branch-runs/run-audience-branch-technical/nodes/publish_bulletin/outputs/publish/selection.json
```

Run a real failure-recovery workflow against the repository compliance-gated
bulletin example:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/compliance-gated-source/team-update.md"

cat > artifacts/compliance-gated-retry-plan.json <<'EOF'
{"fail_until_attempt":1,"gate_policy":"manual-approval","expected_reviewer_group":"release-managers"}
EOF

cat > artifacts/compliance-gated-publication-gate.json <<'EOF'
{"approved":false,"reviewer":"","reviewer_group":"release-managers"}
EOF

cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json

cargo run -p bijux-dag-cli --bin bijux-dag -- run --json \
  evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json \
  --out artifacts/compliance-gated-runs \
  --run-id compliance-gated-source \
  --input "source_note=${SOURCE_NOTE}" \
  --input "retry_plan=$(pwd)/artifacts/compliance-gated-retry-plan.json" \
  --input "publication_gate=$(pwd)/artifacts/compliance-gated-publication-gate.json"
```

Repair only the failed publication boundary after updating approval:

```bash
cat > artifacts/compliance-gated-publication-gate.json <<'EOF'
{"approved":true,"reviewer":"A. Reviewer","reviewer_group":"release-managers"}
EOF

cargo run -p bijux-dag-cli --bin bijux-dag -- replay --json \
  --source-run-id compliance-gated-source \
  --source-run-root artifacts/compliance-gated-runs \
  --out artifacts/compliance-gated-runs \
  --run-id compliance-gated-repaired \
  --from-node validate_publication_gate
```

Inspect retry evidence and verify the repaired run strictly:

```bash
cat artifacts/compliance-gated-runs/run-compliance-gated-source/nodes/fetch_compliance_gate/attempts.json
cat artifacts/compliance-gated-runs/run-compliance-gated-repaired/nodes/publish_bulletin/outputs/publish/bulletin.md
cargo run -p bijux-dag-cli --bin bijux-dag -- verify --json \
  artifacts/compliance-gated-runs/run-compliance-gated-repaired \
  --strict
```

For the warm-cache run, changed-input comparison, and retained-run attribution
path, use
[`docs/bijux-dag/operations/guides/data-pipeline-workflow.md`](docs/bijux-dag/operations/guides/data-pipeline-workflow.md).

For the container prerequisites, retained output layout, and missing-engine
failure behavior, use
[`docs/bijux-dag/operations/guides/container-packaging-workflow.md`](docs/bijux-dag/operations/guides/container-packaging-workflow.md).

For retained branch decisions, skipped-lane evidence, join-trigger behavior,
and replay stability, use
[`docs/bijux-dag/operations/guides/branching-bulletin-workflow.md`](docs/bijux-dag/operations/guides/branching-bulletin-workflow.md).

For retry evidence, approval-boundary repair, replay input rematerialization,
and strict post-repair verification, use
[`docs/bijux-dag/operations/guides/compliance-gated-bulletin-workflow.md`](docs/bijux-dag/operations/guides/compliance-gated-bulletin-workflow.md).

## Documentation

| Handbook | Use it for |
| --- | --- |
| [Repository handbook](https://bijux.io/bijux-core/bijux-core/) | workspace scope, package ownership, release policy, shared architecture, and repository operations |
| [CLI handbook](https://bijux.io/bijux-core/bijux-cli/) | the `bijux` runtime, app and plugin routing, config behavior, diagnostics, and Python packaging |
| [DAG handbook](https://bijux.io/bijux-core/bijux-dag/) | DAG validation, planning, execution, replay, artifacts, compatibility, and operator workflows |
| [Maintainer handbook](https://bijux.io/bijux-core/bijux-dev/) | repository gates, release verification, docs operations, governance, and evidence collection |

Representative DAG workflow guides:

- [`docs/bijux-dag/operations/guides/file-processing-workflow.md`](docs/bijux-dag/operations/guides/file-processing-workflow.md) for a host-shell artifact workflow
- [`docs/bijux-dag/operations/guides/data-pipeline-workflow.md`](docs/bijux-dag/operations/guides/data-pipeline-workflow.md) for changed-input attribution and retained-run comparison
- [`docs/bijux-dag/operations/guides/branching-bulletin-workflow.md`](docs/bijux-dag/operations/guides/branching-bulletin-workflow.md) for retained branch decisions, skipped lanes, and replay stability
- [`docs/bijux-dag/operations/guides/compliance-gated-bulletin-workflow.md`](docs/bijux-dag/operations/guides/compliance-gated-bulletin-workflow.md) for transient retry evidence, focused replay repair, and strict verification after recovery
- [`docs/bijux-dag/operations/guides/container-packaging-workflow.md`](docs/bijux-dag/operations/guides/container-packaging-workflow.md) for mounted container inputs, retained outputs, and recorded image identity

If you are reading code and need the owning package before the owning command,
start with:

- [`docs/bijux-core/foundation/package-map.md`](docs/bijux-core/foundation/package-map.md)
- [`docs/bijux-dag/packages/index.md`](docs/bijux-dag/packages/index.md)
- [`docs/bijux-cli/packages/index.md`](docs/bijux-cli/packages/index.md)

## Maintainer Workflows

```bash
make help
make docs-check
make dag-test
make dag-contracts
```

This repository keeps public product code, contracts, documentation, and
release proof together so drift becomes a reviewable code change rather than an
undocumented side channel.

## License

Apache-2.0 ([`LICENSE`](https://github.com/bijux/bijux-core/blob/main/LICENSE)).
