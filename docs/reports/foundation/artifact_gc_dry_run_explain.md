# Artifact GC Dry-Run Explain Surface

`bijux-dag-artifacts` exposes:

- `plan_lineage_safe_gc(...)` for deterministic preserve/collect sets
- `explain_lineage_safe_gc(...)` for operator-visible reasoning per artifact

Explain entry fields:

- `artifact_id`
- `action` (`preserve` or `collect`)
- `reason` (lineage-reference rationale)

This keeps retention behavior auditable and prevents silent artifact deletion decisions.

