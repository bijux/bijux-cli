# Runtime Keep Quarantine Delete Review

generated_from: `configs/policy/runtime_broad_surface_ownership.json`

## Decision Summary

- keep:
  - `artifacts/storage/semantic_lineage.rs`
- quarantine:
  - all entries with `decision = quarantine` in runtime broad-surface ownership policy
- delete:
  - none currently; deletion deferred until owner-repo migration completion
