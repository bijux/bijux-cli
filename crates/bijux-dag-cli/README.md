# bijux-dag-cli

`bijux-dag-cli` installs the `bijux-dag` executable. It is the publishable,
user-facing command package for the DAG product.

`bijux-dag` v0.4.0 is a local-first DAG runtime for reproducible workflows
with explicit graph contracts, deterministic execution records, verified
artifacts, cache explanation, and replayable run bundles. This crate is the
installed command surface for that promise.

Install it from crates.io when you want the standalone DAG command surface:

```bash
cargo install bijux-dag-cli
bijux-dag --help
```

## Release Status

- public crate on the `v0.4.0` DAG release line
- installs the stable operator-facing `bijux-dag` binary
- does not promote experimental, simulated, or internal namespaces into the
  default public contract

## Stable Operator Boundary

The supported release boundary is the visible `bijux-dag --help` surface:
`validate`, `plan`, `run`, `replay`, `runs`, `artifact`, `artifact-inspect`,
`diff`, `explain`, `verify`, `doctor`, `cache`, `version`, `commands`, and
`completions`.

Experimental routes remain available by explicit path for repository-owned
workflows, and `bijux-dag commands --lane experimental` is the deliberate
inventory surface for that lane. Simulated and maintainer namespaces require
explicit lane inventory through `bijux-dag commands --lane simulated` or
`bijux-dag commands --lane internal`, plus execution opt-in through
`BIJUX_DAG_ENABLE_SIMULATED=1` or `BIJUX_DAG_ENABLE_INTERNAL=1`.

Container-backed nodes are part of the stable local execution surface. When a
supported engine such as Docker is available on `PATH`, `bijux-dag run`
executes the container step, retains stdout and stderr, and records engine and
image identity in the retained trace. When the engine is missing, the run
fails as an infrastructure error rather than silently degrading to another
execution mode.

Branch-backed workflows are also part of the stable local execution surface.
When a DAG uses a branch node, retained runs record the selected decision, the
skipped lane, and the join-node trigger evaluation so operators can inspect the
execution path directly from run evidence.

## What This Crate Owns

- the `bijux-dag` binary entrypoint
- thin startup wiring, process initialization, and exit mapping
- delegation into `bijux-dag-app` for actual command behavior
- shell completion generation for the installed executable
- lane-scoped command discovery for stable, experimental, simulated, and
  maintainer-only route inventories

## What It Does Not Own

- graph semantics
- runtime execution logic
- artifact persistence rules

If the question is about route behavior rather than process startup, the owning
crate is usually `bijux-dag-app`.

## Representative Workflows

- [Runnable Examples](../../docs/bijux-dag/interfaces/examples/index.md)
  maps the repository-backed hello, file-processing, cache, replay, failure,
  branch, and container proofs to their expected outputs.
- [File Processing Workflow](../../docs/bijux-dag/operations/guides/file-processing-workflow.md)
  demonstrates a host-shell artifact workflow with replay and promotion.
- [Cache Behavior Workflow](../../docs/bijux-dag/operations/guides/cache-behavior-workflow.md)
  demonstrates stable cache verification, explicit-path cache-miss
  explanation, selective invalidation, and corruption refusal.
- [Reproducibility Model](../../docs/bijux-dag/interfaces/reference/reproducibility-model.md)
  explains the retained identity surfaces behind cache verification, replay,
  export bundles, and artifact comparison.
- [Data Pipeline Workflow](../../docs/bijux-dag/operations/guides/data-pipeline-workflow.md)
  demonstrates retained-run comparison and changed-input attribution.
- [Branching Bulletin Workflow](../../docs/bijux-dag/operations/guides/branching-bulletin-workflow.md)
  demonstrates retained branch decisions, skipped lanes, join-trigger evidence,
  and replay stability.
- [Compliance-Gated Bulletin Workflow](../../docs/bijux-dag/operations/guides/compliance-gated-bulletin-workflow.md)
  demonstrates transient retry evidence, focused replay repair, and strict
  verification after recovery.
- [Historical Catalog Backfill Workflow](../../docs/bijux-dag/operations/guides/historical-catalog-backfill-workflow.md)
  demonstrates the current internal backfill lane for partition fanout,
  aggregate summary reporting, failed-partition retry, and explicit handoff
  into retained DAG runs through `bijux-dag` with `BIJUX_DAG_ENABLE_INTERNAL=1`.
- [Scheduled Catalog Refresh Workflow](../../docs/bijux-dag/operations/guides/scheduled-catalog-refresh-workflow.md)
  demonstrates the current internal schedule lane for cron preview, durable
  submission, queue dispatch, and explicit ledger-to-run linkage through
  `bijux-dag` with `BIJUX_DAG_ENABLE_INTERNAL=1`.
- [Container Packaging Workflow](../../docs/bijux-dag/operations/guides/container-packaging-workflow.md)
  demonstrates mounted container inputs, retained outputs, and recorded image
  identity.

## Related links

- [Crate contract](./CONTRACT.md)
- [Crate changelog](./CHANGELOG.md)
- [DAG handbook](https://bijux.io/bijux-core/bijux-dag/)
- [Package docs](https://bijux.io/bijux-core/bijux-dag/packages/bijux-dag-cli/)
