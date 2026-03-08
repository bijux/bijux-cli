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
