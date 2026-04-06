# Bundles And Portability

Bundles transport execution context and evidence. Portability is proven by replay/diff outcomes, not by transport success alone.

## What bundles preserve and what they omit

Bundles preserve:

- graph context,
- run/artifact identity references,
- lineage metadata needed for verification workflows.

Bundles do not automatically preserve:

- backend capability parity,
- timing/resource equivalence,
- external side effects not captured as artifacts.

## Export to import roundtrip

```bash
bijux-dag bundle export --run-id RUN_20260309_120 --out ./exports/run120.bundle
bijux-dag bundle import --bundle ./exports/run120.bundle
bijux-dag inspect run --run-id RUN_20260309_120
bijux-dag replay --run-id RUN_20260309_120
bijux-dag diff run --left RUN_20260309_120 --right RUN_20260309_121
```

Interpretation sequence:

1. export/import proves transfer,
2. inspect proves evidence shape is available,
3. replay/diff proves equivalence class.

## Portability versus backend equivalence

- portability: transferable context can be validated in target environment.
- backend equivalence: target backend can satisfy required capability envelope.

Portability can be valid while strict backend equivalence is not.

## When portability degrades

Degradation signals:

- replay classification becomes `bounded` or `incomplete`,
- capability gaps appear in target backend,
- artifact lineage or payload availability is partial.

Interpret degraded portability as conditional evidence, not release-ready equivalence.

## Next reading

- Backend capability boundaries: [Backend Support](../07-operations/05-backend-support.md)
- Formal artifact transport semantics: [Artifact Model Specification](../06-specification/03-artifact-model.md)
