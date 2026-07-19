---
title: Change Validation
audience: contributors
type: quality
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# Change Validation

Use this page after the focused edit loop is green. It defines the evidence a
reviewer needs for the completed CLI change; it does not repeat local
reproduction commands.

## Classify The Contract

| Changed meaning | Required evidence |
| --- | --- |
| command or flag grammar | parser/routing contract, help output, and compatibility note |
| JSON field, error envelope, or ordering | integration contract, schema or snapshot review, and consumer impact |
| stdout/stderr ownership | real-process integration test covering both streams |
| exit status or error category | process-level assertion tying payload, category, and exit code together |
| plugin manifest, namespace, or lifecycle | plugin contract tests and end-to-end lifecycle proof |
| Python launcher or bridge | Python packaging test and native runtime parity proof |
| module ownership | architecture test demonstrating dependency direction |
| public documentation | behavior proof plus strict documentation build |

Additive output can still be incompatible when consumers reject unknown
fields, compare complete objects, or depend on ordering. “The command still
runs” is not sufficient compatibility evidence.

## Evidence Package

A reviewable validation record states:

- the changed contract in one sentence;
- the focused test that owns that contract;
- broader lanes run and their exact scope;
- generated references or snapshots reviewed;
- compatibility effect for scripts, plugins, Python callers, and operators;
- omitted slow, platform, or release checks.

Use [Testing And Validation](../../bijux-core/operations/testing-and-validation.md)
for the exact meaning of repository test lanes. Typical CLI changes widen from
focused tests to `make test`; documentation changes also require
`make docs-check`. Packaging or publication changes use the release validation
surface rather than claiming ordinary tests prove publishability.

## Snapshot And Reference Changes

Treat generated output as evidence, not as a file that must be made green:

1. regenerate through the owning command;
2. inspect semantic differences;
3. confirm the implementation intentionally changed;
4. update compatibility guidance when callers can observe the difference;
5. commit generator and generated output together only when they express one
   inseparable contract change.

Never overwrite a snapshot first and infer correctness from the resulting pass.

## Stop Conditions

Validation is incomplete when:

- only a broad suite covers a narrow public contract incidentally;
- a changed JSON or error surface has no consumer-impact decision;
- human output was checked but machine output was not;
- documentation describes behavior not exercised by the selected test;
- an omitted lane is hidden behind “all checks passed.”

## Continue Reading

- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
- [Test Strategy](test-strategy.md)
- [Definition Of Done](definition-of-done.md)
- [Local Development](../operations/local-development.md)
