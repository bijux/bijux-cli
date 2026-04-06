# Artifacts

Artifacts are the durable output units you can inspect, compare, and carry across environments.

## Artifact lifecycle in practice

A useful mental model:

1. node produces output,
2. runtime records artifact metadata and lineage,
3. artifact identity is computed,
4. inspect/replay/diff workflows consume that evidence,
5. bundle workflows transport artifact context for portability checks.

## What is an artifact and what is not

Artifact:

- persisted output with identity and lineage,
- suitable for diff and replay analysis.

Not artifact (by default):

- transient temp files,
- ad-hoc debug logs,
- generic metadata entries without output payload meaning.

## Realistic lineage example

```text
extract_orders -> orders_raw.parquet (a_raw)
normalize_orders -> orders_normalized.parquet (a_norm)
revenue_report -> revenue_daily.csv (a_report)
```

If `a_raw` changes, `a_norm` and `a_report` may drift. Lineage links make that drift explainable instead of mysterious.

## Integrity and trust boundaries

What users can trust:

- artifact identity for canonical content comparison under one policy version,
- producer linkage (`run_id`, `node_id`) for attribution.

What users must still verify:

- external side effects not captured as artifacts,
- cross-backend equivalence without replay/diff evidence,
- imported artifact context without provenance verification.

## Next reading

- Formal artifact contract: [Artifact Model Specification](../06-specification/03-artifact-model.md)
- Identity inputs and exclusions: [Artifact Identity Specification](../06-specification/06-artifact-identity.md)
- Portability behavior with bundles: [Bundles And Portability](../03-user-guide/08-bundles-and-portability.md)
