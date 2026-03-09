# Portability

Portability means transferable evidence with verifiable interpretation, not blanket cross-backend parity.

## Portability surfaces

- portable state: run/artifact identity context that can be imported and inspected,
- portable evidence: lineage and diagnostics sufficient for replay/diff interpretation,
- portable bundles: transport artifact that carries required context.

## Bounded portability model

Portability is conditional on:

- shared capability envelope between source and target backends,
- compatible identity/canonicalization policy versions,
- required evidence availability after import.

Without those conditions, portability degrades from strict-equivalent to bounded or non-portable.

## Bundle transfer versus equivalence

- bundle export/import proves transport,
- replay/diff proves behavioral class,
- operator policy decides acceptance.

Transport success alone is not a portability guarantee.

## Failure and downgrade interpretation

Typical downgrade triggers:

- capability gap in target backend,
- missing artifact evidence required for comparison,
- incompatible policy versions.

Interpretation rule:

- strict equivalence only with full comparable evidence,
- bounded-equivalent when limits are explicit and accepted,
- non-portable when comparison trust cannot be established.

## Next reading

- User-facing portability workflow: [Bundles And Portability](../03-user-guide/08-bundles-and-portability.md)
- Backend support classes: [Backend Support](../07-operations/05-backend-support.md)
- Artifact portability contract: [Artifact Model Specification](../06-specification/03-artifact-model.md)
