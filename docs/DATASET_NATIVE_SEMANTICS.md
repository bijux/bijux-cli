# Dataset-native semantics and data product modeling

## Dataset identity and contracts

Datasets use first-class identifiers:
- `DatasetId`
- `DatasetVersionId`

Dataset contracts are distinct from low-level artifact identity and can span multiple artifacts and partitions.

## Dataset model capabilities

- schema contracts and schema references
- logical names and physical bindings
- promotion history
- dataset-level lineage
- partition strategy: time, key, range, custom
- completeness semantics
- freshness and staleness policy
- quality outcomes, score, and acceptance state

## Publication and retention

Dataset publication workflows are independent from artifact materialization details.

Retention and archival policy are dataset-scoped and do not depend on run-level artifact retention rules.

## Immutability classes

- append-only
- versioned snapshot
- mutable pointer
- derived view

## Consumption and replay

Consumption contracts support:
- stable version
- latest approved
- freshness bounded input

Replay can be anchored to dataset references rather than incidental file paths.

## Dataset-artifact mapping and diffing

Mapping records preserve decoupling between dataset contracts and storage internals.

Version diffing provides compatibility analysis between dataset versions.

## Provenance, readiness, and catalog

Dataset provenance reports summarize:
- producers
- consumers
- validation pass rate
- promotions

Readiness gates allow schedules to depend on dataset acceptance state.

Catalog query model supports search by schema, lineage-related ownership, freshness, and quality state.
