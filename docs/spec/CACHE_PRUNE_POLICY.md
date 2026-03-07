# Cache Prune Policy

## Current policy
- Safe mode: remove only entries that fail verification.
- Simulation mode: report candidates without mutating cache.

## Future policy candidates
- age-based pruning
- size-budget pruning
- recency + verification score pruning

No future policy is normative until implemented and contract-tested.
