---
title: Executable Specification Authority
audience: maintainers
type: specification-index
status: internal
owner: bijux-dag-governance
last_reviewed: 2026-07-19
---

# Executable Specification Authority

`docs/spec/` contains repository contracts that are read by tests, maintainer
commands, or release governance. These files are not tutorials and are not part
of the public MkDocs navigation. Their stable paths and normative statements
may be executable interfaces.

## What Belongs Here

A document belongs in this directory when at least one of these conditions is
true:

- a test or maintainer command reads the file or requires its path;
- the document defines a schema, state transition, precedence rule, error
  contract, or compatibility rule enforced by code;
- a release or architecture gate requires reviewers to compare implementation
  behavior against the document;
- the document governs how retained evidence is interpreted.

Explanations aimed at users belong in `docs/bijux-dag/`,
`docs/bijux-cli/`, or `docs/bijux-core/`. Observed results belong in
`docs/reports/`. A proposal that is not enforced does not become a contract
merely by being placed here.

## Contract Families

| Family | Representative authorities | Enforcement surface |
| --- | --- | --- |
| execution | `EXECUTION_ENGINE_CONTRACT.md`, `SCHEDULER_CONTRACT.md`, `STATE_MACHINE_CONTRACT.md` | runtime and maintainer contract tests |
| storage and artifacts | `RUN_DIR_CONTRACT.md`, `STORAGE_CONTRACT.md`, `ARTIFACT_LIFECYCLE.md` | artifact, replay, and import/export tests |
| configuration and errors | `CONFIG_PRECEDENCE_CONTRACT.md`, `ERROR_CONTRACT.md` | parser, application, and CLI tests |
| compatibility | `VERSIONING_MODEL.md`, `SCHEMA_EVOLUTION_RULEBOOK.md`, `MIGRATION_POLICY.md` | release and schema-governance checks |
| trust and evidence | `TEST_TRUST_CONTRACT.md`, `COMPARISON_HARNESS_CONTRACT.md`, `FORMAL_INVARIANTS.md` | evidence and anti-drift commands |
| isolation and security | `SECURITY_MODEL.md`, `CONTAINER_EXECUTION_CONTRACT.md`, `REMOTE_EXECUTION_MODEL.md` | backend and security-isolation tests |

The directory is intentionally flat because existing paths are referenced by
source code, tests, reports, and external review links. Moving a contract is an
interface change: update all consumers in one commit and verify the relevant
contract suite rather than adding redirect copies.

## Editing A Contract

Before changing normative behavior:

1. Identify every source and test that reads or cites the contract.
2. State the invariant and failure behavior precisely; avoid aspirational
   language in a canonical contract.
3. Change the implementation, contract, fixtures, and generated evidence
   together when they describe one behavior.
4. Run the narrow contract tests plus
   `bijux-dev::docs_source_reference_contracts`.
5. Regenerate governed reports and inspect semantic differences before
   committing them.

A contract may describe an unsupported capability only when it states that
limit explicitly. Future direction belongs in the product planning authority,
not in normative language.

## Authority And Conflict

Machine-readable schemas govern serialized shape. These prose specifications
govern behavioral intent enforced by repository checks. Public handbooks
explain the supported reader-facing behavior without duplicating the full
contract.

If implementation and specification disagree, the discrepancy is a defect.
The correct response is to determine the intended behavior and update every
authority and evidence consumer; it is not to weaken a test or label the report
as acceptable drift.
