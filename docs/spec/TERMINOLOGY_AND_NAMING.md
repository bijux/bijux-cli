# TERMINOLOGY AND NAMING

Status: stable
Audience: maintainers
Owner: platform documentation guild

This standalone specification consolidates all relevant contract material for this domain.

## SOURCE: docs/spec/GLOSSARY.md
# Superseded by naming contract

This document is preserved as an appendix reference only.

- Superseded by: [TERMINOLOGY_AND_NAMING_CONTRACT.md](./TERMINOLOGY_AND_NAMING_CONTRACT.md)
- Appendix source: [appendices/terminology/GLOSSARY.md](./appendices/terminology/GLOSSARY.md)


## SOURCE: docs/spec/NAMING.md
# Superseded by naming contract

This document is preserved as an appendix reference only.

- Superseded by: [TERMINOLOGY_AND_NAMING_CONTRACT.md](./TERMINOLOGY_AND_NAMING_CONTRACT.md)
- Appendix source: [appendices/terminology/NAMING.md](./appendices/terminology/NAMING.md)


## SOURCE: docs/spec/NAMING_GUIDELINES.md
# Superseded by naming contract

This document is preserved as an appendix reference only.

- Superseded by: [TERMINOLOGY_AND_NAMING_CONTRACT.md](./TERMINOLOGY_AND_NAMING_CONTRACT.md)
- Appendix source: [appendices/terminology/NAMING_GUIDELINES.md](./appendices/terminology/NAMING_GUIDELINES.md)


## SOURCE: docs/spec/NAMING_PHILOSOPHY.md
# Superseded by naming contract

This document is preserved as an appendix reference only.

- Superseded by: [TERMINOLOGY_AND_NAMING_CONTRACT.md](./TERMINOLOGY_AND_NAMING_CONTRACT.md)
- Appendix source: [appendices/terminology/NAMING_PHILOSOPHY.md](./appendices/terminology/NAMING_PHILOSOPHY.md)


## SOURCE: docs/spec/NAMING_REVIEW_POLICY.md
# Superseded by naming contract

This document is preserved as an appendix reference only.

- Superseded by: [TERMINOLOGY_AND_NAMING_CONTRACT.md](./TERMINOLOGY_AND_NAMING_CONTRACT.md)
- Appendix source: [appendices/terminology/NAMING_REVIEW_POLICY.md](./appendices/terminology/NAMING_REVIEW_POLICY.md)


## SOURCE: docs/spec/TERMINOLOGY_AND_NAMING_CONTRACT.md
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
- Avoid abbreviations unless defined in `docs/reference/NAMING_AUDIT.md`.
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
- `docs/reference/NAMING_AUDIT.md`
- `docs/adr/20260308-VOCABULARY-AND-SCOPE-HONESTY.md`

## Implementation and evidence sources

- Naming and vocabulary checks run with architecture and governance audits in:
  - `crates/bijux-dev-dag/tests`
  - `docs/reference/NAMING_AUDIT.md`
  - `docs/adr/20260308-DOCUMENTATION-TRUTH-POLICY.md`

## SOURCE: docs/spec/TERMINOLOGY_GLOSSARY.md
# Superseded by naming contract

This document is preserved as an appendix reference only.

- Superseded by: [TERMINOLOGY_AND_NAMING_CONTRACT.md](./TERMINOLOGY_AND_NAMING_CONTRACT.md)
- Appendix source: [appendices/terminology/TERMINOLOGY_GLOSSARY.md](./appendices/terminology/TERMINOLOGY_GLOSSARY.md)


## SOURCE: docs/spec/TERMS.md
# Superseded by naming contract

This document is preserved as an appendix reference only.

- Superseded by: [TERMINOLOGY_AND_NAMING_CONTRACT.md](./TERMINOLOGY_AND_NAMING_CONTRACT.md)
- Appendix source: [appendices/terminology/TERMS.md](./appendices/terminology/TERMS.md)


## SOURCE: docs/spec/VOCABULARY_SCOPE_HONESTY_POLICY.md
# Vocabulary and Scope Honesty Policy

## Objective

User-facing names must not imply stronger shipped capability than current evidence demonstrates.

## Rules

1. New user-facing terms must appear in `configs/policy/vocabulary_registry.json`.
2. Deprecated overreaching terms must map to canonical replacements.
3. CLI help and generated operator docs must use canonical terms.
4. New names implying production distributed/control-plane/auth-tenancy capability require explicit evidence links.

## Governance

- terminology consistency contracts: `crates/bijux-dev-dag/tests/vocabulary_scope_honesty_guarantees_contracts.rs`
- terminology suite: `configs/suites/terminology_consistency_verification.json`

## SOURCE: docs/spec/appendices/terminology/GLOSSARY.md
# Glossary

- **DAG**: Directed acyclic graph defining node topology and execution dependencies.
- **Run**: One execution instance of a DAG snapshot and plan.
- **Replay**: Re-execution using prior run evidence and compatibility rules.
- **Cache hit**: Reuse of prior verified outputs under matching fingerprint and policy conditions.
- **Artifact**: Versioned output or metadata object produced/consumed by nodes and runs.
- **Trace**: Structured execution diagnostics and timing evidence for nodes and runs.
- **Manifest**: Structured summary describing run identity, nodes, outputs, and provenance.

## SOURCE: docs/spec/appendices/terminology/NAMING.md
# Naming Conventions

## Goals
- Make the layer and intent obvious from the name.
- Prefer domain-specific names over generic ones.
- Keep names stable to avoid unnecessary churn.

## Crates
- `bijux-dag-core`: spec, validation, canonicalization, fingerprints, resolver (pure).
- `bijux-dag-artifacts`: run directory layout, schemas, read/write helpers.
- `bijux-dag-runtime`: planner + engine, adapters, scheduling, cache.
- `bijux-dag-app`: CLI wiring only (no business logic).
- `bijux-dag-cli`: umbrella CLI (dispatches sub-apps).

## Modules
- Avoid catch-all names like `utils`, `common`, `helpers`, `ops`.
- Modules should map to a domain (e.g., `planner`, `cache`, `adapter`, `artifacts`).
- Keep modules small and focused; split by domain when they grow.

## Banned Names
- `utils`, `common`, `helpers`, `ops`
- Generic `Context` without a prefix (use `RunContext`, `ParseContext`, etc.)
- Generic `Config` without a prefix (use `RuntimeConfig`, `PolicyConfig`, etc.)

## Types
- Prefer explicit names: `RuntimeConfig`, `PolicyConfig`, `RunContext`.
- Avoid ambiguous `Config`, `Context`, `Result` in public APIs.
- Names should encode the layer (e.g., `NodeTrace` in artifacts, `ExecutionPlan` in runtime).

## Fields
- Use snake_case for JSON fields and Rust struct fields.
- Prefer semantic names over generic: `node_fingerprint`, `cache_mode`, `graph_snapshot`.

## Commands
- CLI verbs should be short and consistent: `validate`, `run`, `replay`, `diff`, `explain`.
- Prefer `node` over `inspect` for per-node details.
- Prefer `verify` over `verify-run`.

## Files
- Specs live under `docs/spec/` with `v0.1` in the filename when versioned.
- Architecture and ADRs live under `docs/architecture/` and `docs/ADRs/`.

## SOURCE: docs/spec/appendices/terminology/NAMING_GUIDELINES.md
# Naming guidelines

## Scope

This document defines durable naming rules for modules, commands, files, and public symbols.

## Core rules

- Use domain meaning, not delivery status.
- Use stable nouns for module/file names.
- Prefer explicit capability words over broad marketing words.
- Keep naming consistent across code, tests, fixtures, and docs.
- Do not use transitional labels in normative surfaces.

## Disallowed naming patterns

- speculative lifecycle words (`phase`, `task`, `roadmap`) in runtime code surfaces
- marketing qualifiers (`enterprise`, `ecosystem`, `intelligence`, `productization`) in runtime module names
- ambiguous abbreviations without glossary definitions

## Runtime terminology standard

- `engine`: run lifecycle orchestration
- `scheduler`: ready-queue and ordering decisions
- `state`: run and node state transitions
- `backend`: execution substrate adapters
- `policy`: admission and safety decisions
- `execution`: node execution path

## Artifact terminology standard

- `run directory`: authoritative persisted run record
- `manifest`: run-level summary contract
- `trace`: temporal event stream for attempts and lifecycle
- `outputs index`: normalized output file inventory
- `cache proof`: metadata proving reuse validity

## Scheduler terminology standard

- `readiness`: dependency and selector eligibility
- `tie-break`: deterministic ordering for equal priority
- `fairness`: bounded starvation behavior
- `admission`: queue entry policy gate
- `backfill`: historical replay scheduling path

## Rename discipline

When a symbol is renamed:

- update imports and exports in same change
- rename affected tests and fixtures in same change
- update normative docs in same change
- add old-to-new mapping to the naming audit record

## SOURCE: docs/spec/appendices/terminology/NAMING_PHILOSOPHY.md
# Naming philosophy

Names are long-lived contracts.

A good name must survive structural refactors, personnel changes, and release cycles. Naming is treated as a correctness concern because it shapes module boundaries, ownership, and operator expectations.

## Principles

- encode semantics, not ambition
- optimize for future readers over current authors
- avoid transient project-management vocabulary in product surfaces
- avoid overloaded terms across runtime, artifacts, and scheduler domains

## Decision test

A name is acceptable when:

- it describes behavior without release-context knowledge
- it remains accurate under foreseeable implementation changes
- it can be mapped to one glossary entry or one contract section

## SOURCE: docs/spec/appendices/terminology/NAMING_REVIEW_POLICY.md
# Naming review policy

## Review gate

All changes introducing new normative names must satisfy:

- naming rules in `docs/spec/NAMING_GUIDELINES.md`
- glossary alignment in `docs/spec/TERMINOLOGY_GLOSSARY.md`
- audit mapping updates in `docs/reference/NAMING_AUDIT.md` when renaming

## Reviewer checklist

- name reflects stable behavior
- no transitional lifecycle wording in normative surfaces
- no banned marketing qualifiers in runtime module names
- tests/fixtures/docs updated with renamed symbols

## Ownership

- Runtime naming owner: runtime maintainers
- Artifact naming owner: artifact maintainers
- Governance naming owner: dev control-plane maintainers

## SOURCE: docs/spec/appendices/terminology/TERMINOLOGY_GLOSSARY.md
# Terminology glossary

- `attempt`: one execution try for a node.
- `authoritative data`: source-of-truth run metadata and artifacts.
- `cache hit proof`: evidence bundle validating cache reuse.
- `control plane`: repo verification and governance command layer.
- `dag`: directed acyclic graph of node dependencies.
- `effective graph`: normalized graph after defaults and canonicalization.
- `effective plan`: executable plan after lowering and dependency resolution.
- `extension catalog`: registry of extension points and extension descriptors.
- `node result`: canonical node execution outcome value.
- `run directory`: persisted run artifacts and metadata root.
- `sacred flow`: canonical execution path used by run and replay.

## SOURCE: docs/spec/appendices/terminology/TERMS.md
# Terms

- **artifact**: A file or directory produced by a run, stored under the run directory.
- **run**: A single execution of a DAG that produces a run directory and artifacts.
- **node**: An operation in the DAG with inputs, outputs, and parameters.
- **executor**: A runtime component that executes a node.
- **fingerprint**: A deterministic SHA256 hash of canonical node or graph specs.
- **effect**: A declared side effect (filesystem, network, env) required by a node.
