# Platform Scope Name Overstatement Report

Names that overstate shipped capability are tracked as quarantined or speculative surfaces.

| Name family | Current status | Canonical framing |
| --- | --- | --- |
| control plane API | quarantined/speculative | repository governance and diagnostics |
| federated scheduling | speculative | modeled scheduling semantics |
| geo federation | speculative | modeled geo semantics |
| HA scheduler | speculative | modeled availability semantics |
| auth identity/authz/tenancy | speculative or support-only | policy model surfaces |

Governance anchors:
- `configs/policy/runtime_broad_surface_ownership.json`
- `configs/policy/vocabulary_registry.json`
