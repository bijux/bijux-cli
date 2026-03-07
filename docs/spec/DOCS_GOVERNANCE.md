# Documentation governance

## Allowed documentation taxonomy

All normative documentation must live under one of:

- `docs/spec/`
- `docs/architecture/`
- `docs/user/`
- `docs/dev/`
- `docs/reference/`
- `docs/tracking/`

`docs/generated/` is reserved for generated artifacts only.

## Root-doc budget

Root-level markdown files under `docs/` are capped at **110**.

## Required governance documents

- `docs/spec/WORKSPACE_CONTRACT.md`
- `docs/spec/BOUNDARY_RULES.md`
- `docs/spec/EVIDENCE_MODEL.md`
- `docs/spec/DOCS_GOVERNANCE.md`
- `docs/tracking/DOC_OWNERSHIP.json`

## Templates

Contract docs must include:

- scope
- authority
- invariants
- allowed changes
- related tests
- related schemas

Architecture docs must include:

- purpose
- boundaries
- dependencies
- failure modes
- non-goals

User docs must include:

- audience
- prerequisites
- examples
- outputs
- failure behavior

## Content rules

- marketing maturity language is disallowed unless historically quoted
- unsupported guarantee language is disallowed without evidence links
- stale crate names and legacy paths are disallowed
- speculative roadmap content must live in `docs/tracking/`

## Ownership

Normative docs require ownership metadata in `docs/tracking/DOC_OWNERSHIP.json`.
