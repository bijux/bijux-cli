# Authoring Evidence

Use authoring evidence for onboarding-safe DAG graphs, validation negatives,
and instructional examples that are supposed to stay executable.

## What Lives Here

- `minimal/`: the smallest normative graph shapes used in first-run guides and
  command walkthroughs
- `patterns/`: reference topologies for common authoring patterns
- `negative/`: invalid graphs tied to stable validation rule identifiers
- `examples/`: larger runnable examples used by docs and demonstrations

## Ground Rules

- Authoring evidence is for instructional authoring behavior, not battle proof workflows.
- Assets must remain human-readable JSON and avoid speculative unsupported features.
- Every asset declares expected validation and lowering behavior plus command surfaces.
- Docs may only reference authoring assets that exist under `evidence/authoring/`.

## Representative Examples

- `examples/file-processing-report.dag.json`: host-shell artifact workflow with promotable output.
- `examples/regional-sales-pipeline.dag.json`: structured data workflow with changed-input attribution.
- `examples/audience-branch-bulletin.dag.json`: branch-backed bulletin workflow with retained branch decisions, skipped lanes, and replay stability.
- `examples/compliance-gated-bulletin.dag.json`: approval-gated bulletin workflow with retry evidence, repairable approval failure, and focused replay.
- `examples/scheduled-catalog-refresh.dag.json`: schedule-ready bulletin workflow with a required scheduled timestamp, retained request capture, and promotable publication output.
- `examples/historical-catalog-backfill.dag.json`: backfill-ready partition workflow with required window metadata, retained partition capture, and promotable publication output.
- `examples/release-note-bundle.dag.json`: container-backed packaging workflow with mounted inputs, retained outputs, and recorded container identity.
