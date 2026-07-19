# `bijux-dag-runtime` Contracts

`bijux-dag-runtime` owns effectful DAG execution. It turns validated,
planner-ready graph data into node attempts, runtime state transitions,
retained traces, cache decisions, and replay outcomes.

## Owned Surface

The crate owns:

- execution engine and scheduler behavior;
- node readiness, attempt, retry, timeout, cancellation, and terminal state;
- runtime configuration and policy evaluation;
- backend capability negotiation and execution;
- adapter registration, invocation, and result normalization;
- cache eligibility, lookup, write, and explanation;
- replay eligibility and runtime semantic comparison;
- trace emission and artifact-persistence orchestration;
- runtime diagnostics and invariant enforcement.

It does not own graph validity, serialized artifact formats, command routing,
human rendering, or repository governance.

## Internal Boundaries

| Path | Responsibility |
| --- | --- |
| `../src/runtime_core/` | engine, scheduler, state machine, execution context, and invariants |
| `../src/backend/` | backend contracts, capability descriptions, and execution implementations |
| `../src/adapters/` | adapter API, registry, conformance, and built-in adapter boundary |
| `../src/policy/` | runtime policy models, decisions, and traces |
| `../src/cache/` | cache identity, validation, storage, and explanation |
| `../src/replay/` | replay eligibility and semantic comparison |
| `../src/artifacts/` | orchestration over `bijux-dag-artifacts` persistence APIs |
| `../src/diagnostics/` | runtime event, timeline, invariant, and failure diagnostics |
| `../src/internal/` | non-public clocks, selectors, IO, and test support |
| `../src/simulated_platform.rs` | explicit non-stable modeled platform surface |

The crate root curates stable, prelude, and feature-gated experimental exports.
Internal module reachability is not a support promise.

## Execution State Contract

Each node moves through governed states. A transition is accepted only when its
preconditions hold, is recorded once, and yields a deterministic classification
for the same retained inputs and outcomes.

The runtime must preserve:

- dependency and trigger decisions;
- effective runtime configuration and policy;
- backend and adapter identity;
- attempt number, timestamps supplied by the runtime clock, and terminal result;
- artifact and cache identities;
- causal failure and retry classification.

A process exit alone is insufficient evidence of a valid node result. Required
outputs, policy, timeout, cancellation, and persistence outcomes also affect
the terminal classification.

## Effect Boundary

Subprocesses, network calls, clocks, environment access, filesystem operations,
and backend clients must be reached through explicit runtime boundaries.
Planning and policy decisions must not depend on unrecorded ambient values.

Secrets may be supplied to execution but must not enter cache identity,
diagnostic output, traces, or retained command material in clear text.

## Cache And Replay Contract

Cache use is an explained policy decision, not a shortcut around execution
contracts. A hit requires compatible graph, node, input, execution, backend,
adapter, and policy identity according to the cache authority.

Replay requires sufficient compatible source evidence. Missing, corrupt, or
incompatible evidence produces a refusal with reasons; it must not silently
fall back to a fresh run while being reported as replay.

## Backend Boundary

A backend advertises explicit capability support. Unsupported behavior is
refused or classified as modeled; it is not approximated as supported.
Backend-specific status is normalized into runtime-owned result types without
discarding backend identity or diagnostic context.

The simulated platform module is excluded from the stable root and cannot
support production-readiness claims.

## Dependency Direction

The runtime depends on `bijux-dag-core` for graph meaning and
`bijux-dag-artifacts` for retained evidence. `bijux-dag-app` may orchestrate the
runtime. The runtime must not depend on application, CLI, testkit, or maintainer
packages.

## Failure Contract

Failures distinguish validation handed in by the caller, policy refusal,
backend capability, launch, timeout, cancellation, node exit, missing output,
artifact persistence, cache integrity, and replay incompatibility.

Recovery cannot overwrite the original causal record. Retries append attempts;
repair and resume decisions remain explicit and reviewable.

## Verification

| Claim | Required evidence |
| --- | --- |
| engine and scheduler correctness | engine correctness and runtime scheduler/state-machine contracts |
| node execution modes | node execution mode and runtime node execution contracts |
| backend and adapter behavior | adapter backend, runtime, conformance, and reference contracts |
| cache semantics | cache, cache evolution, policy-cache, and runtime-cache contracts |
| replay semantics | replay, runtime replay, and replay determinism contracts |
| policy behavior | runtime policy contracts |

Use focused tests for a bounded change. Broad execution-semantic changes
require the package suite and the relevant application integration contracts:

```bash
cargo test --locked -p bijux-dag-runtime
```

Normative execution, backend, cache, replay, and state-machine authorities live
under `docs/spec/`. This page defines package ownership and proof expectations.
