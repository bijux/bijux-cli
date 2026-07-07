# Authoring Evidence

Purpose: onboarding-safe, executable authoring truth.

Authoring evidence role:
- `minimal`: smallest normative graph shape used for first-run onboarding and command walkthroughs.
- `patterns`: normative reference patterns for common graph topologies.
- `negative`: normative invalid graphs tied to stable validation rule IDs.
- `examples`: illustrative, larger executable examples for docs and demos.

Rules:
- Authoring evidence is for instructional authoring behavior, not battle proof workflows.
- Assets must remain human-readable JSON and avoid speculative unsupported features.
- Every asset declares expected validation and lowering behavior plus command surfaces.
- Docs may only reference authoring assets that exist under `evidence/authoring/`.

Subdirectories:
- `examples/`
- `patterns/`
- `negative/`

Representative examples:
- `examples/file-processing-report.dag.json`: host-shell artifact workflow with promotable output.
- `examples/regional-sales-pipeline.dag.json`: structured data workflow with changed-input attribution.
- `examples/audience-branch-bulletin.dag.json`: branch-backed bulletin workflow with retained branch decisions, skipped lanes, and replay stability.
- `examples/compliance-gated-bulletin.dag.json`: approval-gated bulletin workflow with retry evidence, repairable approval failure, and focused replay.
- `examples/scheduled-catalog-refresh.dag.json`: schedule-ready bulletin workflow with a required scheduled timestamp, retained request capture, and promotable publication output.
- `examples/release-note-bundle.dag.json`: container-backed packaging workflow with mounted inputs, retained outputs, and recorded container identity.
