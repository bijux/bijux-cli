# Evidence Family Boundaries

## Canonical Ownership
- battle: release trust workflows and trust-property proofs.
- cache: cache corruption, warm/cold, and replay fixtures.
- compat: support decision fixtures for schema, run directory, and export bundles.
- fault: fault classes and expected system reactions.
- authoring negative: validation-authoring failures only.

## Boundary Rules
- battle may consume cache, compat, and fault assets, but does not own their fixture trees.
- cache replay fixtures are first-class cache subfamily assets under `evidence/cache/replay/`.
- compat assets are not battle assets unless explicit consumer linkage exists.
- fault assets are not authoring negatives.
