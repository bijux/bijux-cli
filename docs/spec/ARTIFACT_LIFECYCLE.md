# Artifact lifecycle

## States

- `staging`: run data is being written.
- `incomplete`: run interrupted before manifest finalization.
- `finalized`: manifest and finalization markers written.
- `retained`: data kept by retention policy.
- `pruned`: removable non-retained data cleaned.

## Guarantees

- Finalized runs are immutable.
- Incomplete runs are explicitly marked.
- Cleanup planning must preserve retained prefixes.
