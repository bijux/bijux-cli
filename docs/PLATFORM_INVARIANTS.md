# Non-negotiable platform invariants

1. Deterministic planning must produce identical plans for identical graph and policy inputs.
2. Artifact content identity is immutable after publication.
3. Tenant isolation boundaries are never bypassed by scheduler, API, or storage paths.
4. Authorization deny decisions are final and cannot be overridden implicitly.
5. Replay and recovery must preserve provenance and audit visibility.
