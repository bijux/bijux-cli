# Advanced artifact intelligence and semantic lineage

## Semantic dependency classes

Lineage tracks dependency intent using:
- data dependency
- control dependency
- quality dependency
- policy dependency

## Confidence and declared-vs-inferred lineage

Lineage confidence distinguishes exact declared lineage from inferred lineage with explicit levels.

## Artifact semantics and relationships

Semantic tags include model, dataset, report, checkpoint, metric bundle, and compliance evidence.

Relationship types include derived-from, validated-by, approved-by, superseded-by, and promoted-from.

## Cross-run and large-history handling

The model supports cross-run stitching for recurring schedules and backfills.

Large histories can be summarized with deterministic drill-down preservation.

## Impact and reverse-impact analysis

Supported analysis surfaces:
- downstream impact (runs, datasets, artifacts)
- upstream trust input influence for a selected result

## Export and materialization

Lineage export formats include JSON, JSON Lines, and GraphML.

Materialization rules allow caching lineage computations with strict invalidation on semantic changes.

## Retention protection and replay guidance

Lineage-aware retention can protect critical ancestry from accidental cleanup.

Replay recommendations provide minimal recomputation sets from semantic upstream dependencies.

## Conflict detection and reconciliation

The system detects inconsistent derivation claims and supports reconciliation plans for imported bundles.

## Quality score and policy hooks

A lineage quality score tracks completeness, exactness, and verification coverage.

Policy hooks can enforce minimum lineage quality for operations that depend on policy-sensitive ancestry.
