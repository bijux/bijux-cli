# Cache semantics

## Cache modes

- `off`: never read/write cache entries.
- `read`: read cache entries when valid; do not write new entries.
- `readwrite`: read cache when valid and write missing/updated cache entries.

## Cache proof contract

Each node trace may include proof fields indicating:
- `hit`: whether a cache entry was used.
- `corrupt_detected`: whether cached data integrity failed and was rejected.
- `validated`: whether fingerprint/provenance checks passed.

## Repair expectations

- Corrupt entries must be detected and recomputed before returning success.
- Cache directory must remain deterministic for repeated runs under unchanged inputs and policies.
- Offline cache checks can fail closed in `read` mode; successful runs must recompute invalid entries.
