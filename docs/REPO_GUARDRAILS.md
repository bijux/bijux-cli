# Repository guardrails

## Source layout

- Required roots: `crates`, `docs`, `examples`, `configs/nextest`
- Guardrail tests live under `crates/bijux-dev-dag/tests`
- Layout contract config: `configs/repo/source_layout_guardrails.toml`

## File size budget

- Source modules should stay below 2200 lines.
- Temporary allowlist entries must be explicit and reduced over time.

## Governance

Guardrails are part of release verification and should be updated only with clear rationale.
