---
title: Quality Policy
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Quality Policy

Use this page when the question is not "did a command run," but "what quality
bar do maintainer tools have to meet before their output can be trusted?"

Maintainer tooling sits in the path of release, governance, and repository
proof. A weak tool can make the repository look healthy when it is not, or
make a real failure harder to understand than it should be.

## Quality Principles

- quality claims require executable evidence
- diagnostics must identify failing surfaces and likely remediation path
- governance commands must be predictable and scriptable
- policy drift must be detected through suites, not manual memory

## Signals Of Healthy Maintainer Tooling

- green required gates for scope of change
- stable machine-readable outputs for automation consumers
- updated docs and policy notes for changed behavior
- readable human output that points to the failing ownership surface

## What The Policy Protects

| Surface | Why quality matters |
| --- | --- |
| shared gates | they decide whether work is releasable or blocked |
| diagnostics | they shape how quickly maintainers find the real owner |
| evidence reports | they influence release and governance decisions |
| automation consumers | they rely on stable output instead of fragile text scraping |

## Failure Modes This Policy Rejects

- a green-looking summary that hides unresolved failures
- machine output that changes shape without an explicit compatibility decision
- commands that pass locally only because they depend on ambient state
- policy checks that rely on maintainer memory instead of executable rules

## Code Anchors

- `crates/bijux-dev/src/commands/command_runtime.rs`
- `crates/bijux-dev/src/suites/check.rs`
- `crates/bijux-dev/src/suites/contract.rs`

## Continue Reading

- [Test Policy](test-policy.md)
- [Dependency Governance](dependency-governance.md)
- [Core Testing and Validation](../../bijux-core/operations/testing-and-validation.md)
