---
title: Dependency Governance
audience: maintainers
type: governance
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-04-06
---

# Dependency Governance

Use this page when a dependency change touches maintainer tooling and the real
question is not "does it compile," but "what part of repository proof might
change with it?"

Dependency governance exists because maintainer dependencies do more than add
code. They can change report shapes, shell behavior, release flows, and the
meaning of green gates.

## Rules

- prefer minimal dependencies with clear ownership rationale
- review transitive impact on command outputs and test behavior
- pin or constrain versions for compatibility-sensitive tooling
- require evidence updates when dependency changes affect policy surfaces

## High-Risk Triggers

- serialization or schema dependencies used in evidence outputs
- tooling dependencies that change shell/process behavior
- dependencies used by release and documentation pipelines

## What Reviewers Should Check

| Change type | Why it is risky |
| --- | --- |
| output-shape dependency change | automation or release proof may read different data |
| process or shell dependency change | command semantics can shift without obvious code changes |
| docs or release pipeline dependency change | publication or handbook integrity can drift from local expectations |

## What This Page Is Not Saying

- It is not banning new dependencies outright.
- It is not replacing cargo-level review of the actual manifests.
- It is not saying every version bump needs the same depth of follow-up.

## Code Anchors

- `crates/bijux-dev/Cargo.toml`
- `crates/bijux-dev/src/tooling/`
- `crates/bijux-dev/src/commands/shared_io.rs`

## Continue Reading

- [Quality Policy](quality-policy.md)
- [Security and Secrets](security-and-secrets.md)
- [Core Decision Record Policy](../../bijux-core/governance/decision-record-policy.md)
