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
