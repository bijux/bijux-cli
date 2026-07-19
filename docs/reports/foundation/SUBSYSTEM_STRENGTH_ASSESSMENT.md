---
title: Subsystem Strength Assessment Report
audience: maintainer
type: report
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-19
---

# Subsystem Strength Assessment Report

## Interpretation

“Strength” means the subsystem has a named owner, explicit boundary, executable
evidence, and honest limitation route. It does not mean defect-free, complete,
or suitable for every deployment. This assessment describes governance
coverage at the reviewed source revision; it is not a substitute for running
the linked checks.

## Assessment Scale

| Classification | Meaning |
| --- | --- |
| contract-backed | authority and focused executable evidence are present |
| conditional | evidence exists, but support depends on backend, environment, or declared release lane |
| internal | repository-supported for development or verification, not a public product commitment |
| incomplete | owner, contract, executable evidence, or limitation route is missing |

## Subsystem Assessment

| Subsystem | Classification | Evidence route | Residual boundary |
| --- | --- | --- | --- |
| `bijux` routing, config, plugins, and structured output | contract-backed | CLI routing, architecture, and integration suites | plugin execution is trust-based rather than sandboxed |
| Python `bijux` distribution | contract-backed | packaging, launcher parity, and Python test suites | it does not embed the DAG runtime |
| graph validation, identity, and planning | contract-backed | DAG core contracts, authoring parity, and planner suites | runtime effects remain out of scope |
| run artifact layout, integrity, and lineage | contract-backed | artifact hardening, conformance, and import/export suites | backend durability is capability-dependent |
| local execution, cache, replay, and scheduling | contract-backed | runtime state, cache, replay, cancellation, and evidence suites | correctness claims require retained run verification |
| container, Kubernetes Job, and shared-filesystem SLURM lanes | conditional | backend contracts and owned workflow evidence | availability and isolation depend on the selected engine or cluster |
| DAG application and command response contracts | contract-backed | app route, output schema, and executable recipe suites | modeled and internal routes remain outside the stable lane |
| testkit fixtures and assertions | internal | deterministic fixture and harness contracts | not a public crates.io dependency |
| maintainer suites and governed reports | internal | command-surface, evidence-access, and governance contracts | a generated report is not proof until its command finishes successfully |
| public documentation | contract-backed | strict MkDocs build, source references, navigation, and publication budget | shared-standard checksum drift must be resolved through `bijux-std` |

## Review Rule

A classification can advance only through a change to authority and executable
evidence, not through prose. A failing check, missing fixture, unsupported
backend, or stale generated report must remain visible in the final assessment.
If evidence is not run for the reviewed revision, report “not verified” rather
than carrying forward an earlier pass.

## Cross-Subsystem Risks

- command presentation can overstate an internal or conditional runtime lane;
- generated references can drift from parser, schema, or manifest authority;
- shared fixtures can hide package ownership when they encode product policy;
- concurrent tests can corrupt evidence when they share checkout paths;
- report inventories can look complete while their producers or final statuses
  are absent.

Foundation review should route each risk to the owning contract and focused
gate instead of adding a broad exception to this report.
