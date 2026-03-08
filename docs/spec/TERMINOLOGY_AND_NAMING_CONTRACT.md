# Terminology and naming contract

**What this spec is not**: user tutorial, glossary expansion, marketing language, or workflow cookbook.

## Scope

This contract is the single source of meaning and naming rules for contract terms used across
`spec/`, `reference/`, `architecture/`, and operator-facing command surfaces.

## Canonical vocabulary

| Term | Meaning | Reference |
| --- | --- | --- |
| graph | Deterministic computation graph and operator surface identity unit | `docs/spec/GRAPH_IDENTITY_CONTRACT.md` |
| run | Executed workflow instance with recorded planning/execution artifacts | `docs/spec/RUN_HISTORY_CONTRACT.md` |
| artifact | Stored immutable output and lineage object | `docs/spec/ARTIFACT_LIFECYCLE.md` |
| replay | Deterministic re-execution workflow that preserves semantic intent and diagnostics | `docs/spec/REPLAY_CONTRACT.md` |
| lineage | Provenance chain for graphs, runs, and artifacts | `docs/spec/ARTIFACT_LINEAGE_COMPLETENESS_CONTRACT.md` |
| scheduler | Runtime node orchestration engine with bounded deterministic policy | `docs/spec/SCHEDULER_CONTRACT.md` |
| evidence | Executable output used to justify claims and release trust | `docs/spec/EVIDENCE_GLOSSARY.md` |

## Naming conventions

- Use nouns for durable concepts (contract, contract surfaces, contracts families).
- Avoid abbreviations unless defined in `docs/architecture/naming_audit.md`.
- Keep names stable once documented in a canonical contract.
- Favor surface-reflective naming for command families and registry fields.

## Naming rule enforcement

- Stable names appear in command taxonomies and migration matrices in lockstep.
- New naming terms need either:
  - explicit migration policy and evidence, or
  - a scoped experimental boundary mark with expiry.

## Governance references

- canonical naming details: `docs/spec/appendices/terminology/NAMING.md`
- naming constraints and phrase policy: `docs/spec/appendices/terminology/NAMING_GUIDELINES.md`
- philosophical rationale: `docs/spec/appendices/terminology/NAMING_PHILOSOPHY.md`
- naming review governance: `docs/spec/appendices/terminology/NAMING_REVIEW_POLICY.md`
- term surface and glossary: `docs/spec/appendices/terminology/TERMINOLOGY_GLOSSARY.md`
- canonical glossary table: `docs/spec/appendices/terminology/GLOSSARY.md`
- term index: `docs/spec/appendices/terminology/TERMS.md`
- `docs/architecture/naming_audit.md`
- `docs/adr/20260308-vocabulary-and-scope-honesty.md`

## Implementation and evidence sources

- Naming and vocabulary checks run with architecture and governance audits in:
  - `crates/bijux-dev-dag/tests`
  - `docs/architecture/naming_audit.md`
  - `docs/adr/20260308-documentation-truth-policy.md`
