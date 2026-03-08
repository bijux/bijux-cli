# ADR: Dev DAG Cleanup End-State

Date: 2026-03-08
Status: accepted

## Context
`bijux-dev-dag` has grown as the repository control-plane. The cleanup objective is to keep it authoritative for governance orchestration without absorbing runtime semantics ownership.

## Decision
The end-state for `bijux-dev-dag` is:
- command orchestration for policy, evidence, and release checks
- report generation for governance visibility
- zero ownership of runtime semantic source-of-truth models
- explicit classification of evidence commands into release-critical and advisory sets
- enforced test lane behavior for blocking vs non-blocking evidence outcomes

## Consequences
- New governance surfaces must attach to existing ownership/policy docs.
- Runtime semantic changes remain owned by runtime/core contracts, not dev control-plane helpers.
- `make test-all` remains the release-critical evidence gate for CI.
