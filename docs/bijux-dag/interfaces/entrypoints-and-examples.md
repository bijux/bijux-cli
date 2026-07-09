---
title: Entrypoints and Examples
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Entrypoints and Examples

This page is the practical starting point for `bijux-dag` users who want
concrete commands instead of conceptual overviews.

If you want the repository-backed example set indexed by workflow and expected
output instead of by interface surface, start with
[Runnable Examples](examples/index.md).

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles.

The CLI examples on this page stay on the stable `v0.4.0` operator surface
from the [Release Boundary](../foundation/release-boundary.md), except where a
section explicitly calls out an experimental route.

## Proof Map

Use this page when you want the product claims tied to the first commands and
artifacts you can actually inspect:

- explicit graph contracts are proven by `validate` before any run starts
- deterministic execution records are proven by retained node traces under `artifacts/`
- verified artifacts are proven by `artifact registry` and `artifact-inspect`
- cache explanation is proven by the cache-behavior workflow
- replayable run bundles are proven by the reproducibility model for replay identity

## Choose An Entry Point

| If you want to... | Start here |
| --- | --- |
| validate and run a simple DAG | [File-processing commands](#file-processing-cli-entrypoint) |
| inspect artifacts from a retained run | [File-processing commands](#file-processing-cli-entrypoint) |
| compare two runs of a structured workflow | [Structured workflow comparison](#structured-workflow-comparison) |
| verify cache behavior and cache-miss explanation | [Cache-oriented commands](#cache-oriented-commands) |
| test branch decisions and replay stability | [Evidence-backed branch workflow](#evidence-backed-branch-workflow) |
| test retry, failure attribution, and focused repair | [Failure-recovery workflow](#failure-recovery-workflow) |
| embed a minimal DAG parse path in Rust | [Rust entrypoint example](#rust-entrypoint-example) |

## CLI Entrypoints

### File-Processing CLI Entrypoint

```bash
SOURCE_DIR="$(pwd)/evidence/dag/authoring/examples/file-processing-source"
bijux-dag validate evidence/dag/authoring/examples/file-processing-report.dag.json
bijux-dag run evidence/dag/authoring/examples/file-processing-report.dag.json \
  --out ./artifacts/file-processing-runs \
  --run-id file-processing-source \
  --cache readwrite \
  --cache-dir ./artifacts/file-processing-cache \
  --input "source_dir=${SOURCE_DIR}"
bijux-dag artifact-inspect \
  ./artifacts/file-processing-runs/run-file-processing-source \
  render_report:report.md
bijux-dag replay --source-run-id file-processing-source \
  --source-run-root ./artifacts/file-processing-runs \
  --out ./artifacts/file-processing-runs \
  --run-id file-processing-rerun \
  --from-node render_report
bijux-dag diff \
  ./artifacts/file-processing-runs/run-file-processing-source \
  ./artifacts/file-processing-runs/run-file-processing-rerun \
  --mode semantic --explain
```

That single sequence covers the basic spine of the shipped local operator
surface: validate, run, inspect, replay, and diff.

### Structured Workflow Comparison

For a structured data workflow with changed-input attribution, use the regional
sales example. The retained-run comparison below assumes the warm and updated
runs from the dedicated workflow guide already exist:

```bash
ORDERS_CSV="$(pwd)/evidence/dag/authoring/examples/regional-sales-source/orders.csv"
TARGETS_JSON="$(pwd)/evidence/dag/authoring/examples/regional-sales-source/targets.json"
bijux-dag validate evidence/dag/authoring/examples/regional-sales-pipeline.dag.json
bijux-dag run evidence/dag/authoring/examples/regional-sales-pipeline.dag.json \
  --out ./artifacts/regional-sales-runs \
  --run-id regional-sales-cold \
  --cache readwrite \
  --cache-dir ./artifacts/regional-sales-cache \
  --input "orders_csv=${ORDERS_CSV}" \
  --input "targets_json=${TARGETS_JSON}" \
  --input "report_title=Regional Revenue Attainment"
bijux-dag runs compare regional-sales-warm regional-sales-updated \
  --root ./artifacts/regional-sales-runs \
  --json
```

### Cache-Oriented Commands

For the same repository workflow when the question is cache behavior rather
than retained-run comparison, use the cache guide surfaces directly:

```bash
bijux-dag --json why-cache-missed \
  --run-dir ./artifacts/regional-sales-runs/run-regional-sales-updated \
  --node clean_orders \
  --cache-dir ./artifacts/regional-sales-cache

bijux-dag --json cache verify \
  --cache-dir ./artifacts/regional-sales-cache
```

That path stays honest about the current release boundary: `cache verify` is on
the stable operator surface, while `why-cache-missed` is repository-tested but
still an explicit-path experimental diagnostic route in `v0.4.0`.

### Evidence-Backed Branch Workflow

For one retained workflow family that combines branch routing, warm cache
reuse, changed-run comparison, replay proof, strict verification, and final
promotion, use the evidence-backed bulletin workflow:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/audience-branch-source/team-update.md"
bijux-dag validate evidence/dag/authoring/examples/audience-branch-bulletin.dag.json
bijux-dag run --json evidence/dag/authoring/examples/audience-branch-bulletin.dag.json \
  --out ./artifacts/evidence-backed-bulletin-runs \
  --run-id branch-bulletin-cold \
  --cache readwrite \
  --cache-dir ./artifacts/evidence-backed-bulletin-cache \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=technical"
```

Continue with the dedicated guide for the full retained-run comparison, replay,
verification, and promotion sequence:
[Evidence-Backed Bulletin Workflow](../operations/guides/evidence-backed-bulletin-workflow.md).

### Fastest Local Onboarding

For the fastest repository-backed onboarding path that still proves retained
artifacts, warm cache reuse, focused replay, and strict verification, start
with the first-run tutorial:

```bash
cargo run -p bijux-dag-cli --bin bijux-dag -- version
cargo run -p bijux-dag-cli --bin bijux-dag -- validate \
  evidence/dag/authoring/examples/file-processing-report.dag.json
```

Continue with the full path in
[First-Run Tutorial](../operations/guides/first-run-tutorial.md).

### Container-Backed Workflow

For a real container-backed packaging workflow, validate and run the release
note example with one path input and one graph-owned label:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/release-note-source/weekly-update.md"
bijux-dag validate evidence/dag/authoring/examples/release-note-bundle.dag.json
bijux-dag run --json evidence/dag/authoring/examples/release-note-bundle.dag.json \
  --out ./artifacts/release-note-bundle-runs \
  --run-id release-note-bundle \
  --input "source_note=${SOURCE_NOTE}" \
  --input "bundle_label=Release Brief"
cat ./artifacts/release-note-bundle-runs/run-release-note-bundle/nodes/package_bundle/trace.json
```

### Minimal Branch Inspection Workflow

For a real branch-backed workflow, validate and run the audience-routing
example with one path input and one enum branch selector:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/audience-branch-source/team-update.md"
bijux-dag validate evidence/dag/authoring/examples/audience-branch-bulletin.dag.json
bijux-dag run --json evidence/dag/authoring/examples/audience-branch-bulletin.dag.json \
  --out ./artifacts/audience-branch-runs \
  --run-id audience-branch-technical \
  --input "source_note=${SOURCE_NOTE}" \
  --input "audience_mode=technical"
cat ./artifacts/audience-branch-runs/run-audience-branch-technical/nodes/choose_audience_lane/trace.json
```

### Failure-Recovery Workflow

For a real failure-recovery workflow, validate the compliance-gated bulletin
example, allow one transient retry, and repair only the failed publication
boundary after approval changes:

```bash
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/compliance-gated-source/team-update.md"
cat > ./artifacts/compliance-gated-retry-plan.json <<'EOF'
{"fail_until_attempt":1,"gate_policy":"manual-approval","expected_reviewer_group":"release-managers"}
EOF
cat > ./artifacts/compliance-gated-publication-gate.json <<'EOF'
{"approved":false,"reviewer":"","reviewer_group":"release-managers"}
EOF
bijux-dag validate evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json
bijux-dag run --json evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json \
  --out ./artifacts/compliance-gated-runs \
  --run-id compliance-gated-source \
  --input "source_note=${SOURCE_NOTE}" \
  --input "retry_plan=$(pwd)/artifacts/compliance-gated-retry-plan.json" \
  --input "publication_gate=$(pwd)/artifacts/compliance-gated-publication-gate.json"
cat > ./artifacts/compliance-gated-publication-gate.json <<'EOF'
{"approved":true,"reviewer":"A. Reviewer","reviewer_group":"release-managers"}
EOF
bijux-dag replay --json --source-run-id compliance-gated-source \
  --source-run-root ./artifacts/compliance-gated-runs \
  --out ./artifacts/compliance-gated-runs \
  --run-id compliance-gated-repaired \
  --from-node validate_publication_gate
```

## Rust Entrypoint Example

```rust
use bijux_dag_core::parse_graph_strict;

let graph = parse_graph_strict("{\"spec\":\"bijux-dag/v0.1\",\"nodes\":[],\"edges\":[]}")?;
println!("spec={}", graph.spec);
```

## Code Anchors

- `crates/bijux-dag-cli/src/main.rs`
- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dag-core/src/lib.rs`
- `crates/bijux-dag-runtime/src/lib.rs`

## Next Reads

- [Runnable Examples](examples/index.md)
- [CLI Surface](cli-surface.md)
- [Generated CLI Reference](generated-cli-reference.md)
- [Gated Command Inventory](reference/gated-command-inventory.md)
- [Operator Workflows](operator-workflows.md)
- [First-Run Tutorial](../operations/guides/first-run-tutorial.md)
- [Evidence-Backed Bulletin Workflow](../operations/guides/evidence-backed-bulletin-workflow.md)
- [Branching Bulletin Workflow](../operations/guides/branching-bulletin-workflow.md)
- [Compliance-Gated Bulletin Workflow](../operations/guides/compliance-gated-bulletin-workflow.md)
- [Container Packaging Workflow](../operations/guides/container-packaging-workflow.md)
- [Cache Behavior Workflow](../operations/guides/cache-behavior-workflow.md)
- [Data Pipeline Workflow](../operations/guides/data-pipeline-workflow.md)
- [File Processing Workflow](../operations/guides/file-processing-workflow.md)
- [Historical Catalog Backfill Workflow](../operations/guides/historical-catalog-backfill-workflow.md)
- [Scheduled Catalog Refresh Workflow](../operations/guides/scheduled-catalog-refresh-workflow.md)
- [Local Development](../operations/local-development.md)
