# Runtime Quarantine Rationale

The repository keeps several broad platform surfaces in-tree for contract continuity and evidence history, but these surfaces are quarantined from kernel-stable claims.

## Why Quarantine Instead of Immediate Removal

- preserve existing evidence and compatibility contracts during scope contraction
- prevent accidental breakage of release governance checks that still reference modeled surfaces
- maintain explicit owner mapping for migration into dedicated repositories
- keep kernel/runtime execution boundaries explicit while migration proceeds
