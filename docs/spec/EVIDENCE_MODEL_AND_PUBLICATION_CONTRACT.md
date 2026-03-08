# Evidence model, publication, and governance contract

**What this spec is not**: benchmark execution playbooks, release engineering procedure, or implementation internals.

## Scope

This contract is the canonical source for:

- evidence vocabularies and trust claims
- evidence publication and release-claim gating
- internal-only evidence boundaries
- audit report and registry access contracts

## Canonical principles

- Evidence claims require reproducible artifacts and traceable source links.
- Internal diagnostic surfaces are not suitable as release evidence.
- Vocabulary consistency is required across tests, docs, and governance surfaces.
- Audit findings must differentiate unsupported approximations from implemented guarantees.

## Canonical evidentiary model

Refer to appendix sections for:

- model terms and glossary
- publication quality and trust lanes
- internal-only and public evidence separation
- audit contract and access controls

## Implementation and evidence sources

- Evidence registry and fixtures under `evidence/`.
- Verification workflows in `crates/bijux-dev-dag`.
- Evidence contracts and testkit access surfaces in `crates/bijux-dag-testkit`.
