# Root Messaging Contract

## Scope

This contract governs root-level messaging surfaces (`README.md`, root docs, and release-facing summary text).

## Invariants

- Root one-liner must be exactly: `Git for computation graphs.`
- Root mission wording must align with `docs/spec/MISSION_STATEMENT.md`.
- Root docs must not imply execution support beyond `docs/reference/EXECUTION_SUPPORT_POLICY.md`.
- Experimental or simulated behavior must be explicitly labeled.
- Alternative drifting taglines are disallowed in root messaging.

## Oversell guardrails

The following claim patterns are disallowed in root messaging unless linked to conformance evidence:

- "full platform"
- "production-ready distributed orchestration"
- "drop-in replacement for Airflow"
- "drop-in replacement for Prefect"
- "drop-in replacement for Dagster"

## Related tests

- `crates/bijux-dev-dag/tests/root_messaging_contracts.rs`
- `crates/bijux-dev-dag/tests/release_evidence_linkage_contracts.rs`
