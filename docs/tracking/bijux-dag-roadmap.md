---
title: Bijux Dag Roadmap
audience: mixed
type: explanation
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Bijux Dag Roadmap

`bijux-dag` ships `v0.4.0` as a serious local DAG runtime and CLI. This page
explains what comes after that release without blurring future direction into a
shipped contract.

Use [Release Boundary](../bijux-dag/foundation/release-boundary.md) for the
current public promise and [Known Limitations](../bijux-dag/quality/known-limitations.md)
for the current constraints. Use this roadmap only when the question is what
must happen before a later release can honestly claim more.

## Reading Rule

- treat this page as product direction, not as proof that a future surface is
  already stable
- promote later surfaces only when docs, tests, retained evidence, and release
  boundaries all agree
- keep local-first execution trustworthy while the product grows into richer
  graph, scheduling, and backend work

## Release Ladder

| Release lane | What it should add | What it must not pretend to be |
| --- | --- | --- |
| `v0.4.x` hardening | stronger local reliability, sharper evidence, clearer operator recovery | a distributed scheduler or cluster platform |
| `v0.5` graph expressiveness | richer graph authoring and clearer fanout, join, and reuse semantics | a promise that every authoring convenience is already portable everywhere |
| `v0.6` scheduling and backfill | promoted schedule and backfill workflows with durable evidence and operator rules | a public always-on scheduler service without proven boundaries |
| `v0.7` remote workers | deliberate remote execution boundaries and worker lifecycle semantics | generic distributed orchestration parity |
| `v0.8` HPC and Kubernetes | explicit cluster and batch backend integrations with backend-specific contracts | one abstract backend that hides real environment differences |
| `v1.0` stable API | a stability promise across stable CLI, retained evidence, and supported library lanes | a guarantee that every repository route or experiment is frozen forever |

## v0.4.x Hardening

The immediate job after `v0.4.0` is to make the local product harder to
misread and harder to break.

- deepen deterministic evidence, replay, cache, and repair flows
- tighten operator docs so stable, experimental, simulated, internal, and
  future surfaces remain obvious
- strengthen real workflow coverage for failure recovery, retained evidence,
  and environment policy behavior

This lane is intentionally local-first. It does not widen the public product
claim beyond the current concrete backend lanes that already ship in `v0.4.0`.
Remote workers, public scheduling services, broader HPC portability, and
enterprise control planes still belong to later release decisions.

## v0.5 Graph Expressiveness

The next meaningful expansion is richer graph authoring without weakening
determinism.

- make branching, fanout, joins, and aggregation easier to express and easier
  to validate
- improve reusable graph building blocks only when they preserve clear identity
  and replay semantics
- keep schema evolution readable enough that a user can tell why a graph is
  accepted, rejected, or downgraded

This lane is about authoring power, not scheduler claims. More expressive DAGs
still need to execute within a disciplined local runtime before they can become
evidence for later distributed work.

## v0.6 Scheduling and Backfill

Scheduling and backfill deserve their own release lane because the repository
already carries internal evidence for them, but the stable operator contract
does not.

- promote schedule and backfill flows only if durable submission, retry,
  exhaustion, and audit evidence become stable and explainable
- document operator expectations for cron preview, backfill planning, retry
  policy, and retained summary state
- define clear failure boundaries so scheduled work is inspectable instead of
  magical

This is the point where the current internal lane can become a real public
surface, but only if the release boundary, workflow guides, and retained proof
all support the promotion honestly.

## v0.7 Remote Workers

Remote workers are the first release lane that changes the core execution
boundary rather than extending the local controller.

- define worker registration, capability declaration, and lifecycle ownership
- define how inputs, outputs, logs, and replay evidence move across the worker
  boundary
- keep failure attribution explicit so retries, timeouts, and partial results
  remain explainable

This lane should not ship as a vague "distributed mode." It only becomes real
when worker identity, transport assumptions, artifact ownership, and operator
recovery procedures are all concrete.

## v0.8 HPC and Kubernetes

Cluster and batch backends should arrive as explicit backend contracts, not as
marketing shorthand.

- introduce Kubernetes and HPC or batch integrations only with backend-specific
  capability and downgrade rules
- make environment, filesystem, secret, and artifact assumptions visible per
  backend
- keep backend differences inspectable instead of pretending they behave like
  the local runtime in every detail

This lane is where `bijux-dag` can become useful beyond one host, but only by
being more specific about backend truth, not less.

## v1.0 Stable API

`v1.0` should mean that the stable lanes finally deserve a compatibility
promise.

- freeze the stable `bijux-dag --help` operator contract deliberately
- freeze retained run-evidence shapes that operators and tooling are expected
  to consume directly
- identify which Rust library lanes are intended for long-lived public use and
  keep internal compatibility shims out of the primary contract

`v1.0` is not a finish line for all possible features. It is the point where
the shipped surfaces are narrow enough, proven enough, and documented enough to
support a serious long-lived stability promise.

## Promotion Rule

No roadmap lane becomes part of the public product boundary until the release
boundary, workflow docs, package docs, and repository tests all tell the same
story. If those surfaces disagree, the narrower claim wins.
