# `bijux-dag-app` Contracts

`bijux-dag-app` owns operator-facing DAG workflows. It translates command
models into calls to graph, runtime, and artifact packages, then returns typed
responses for machine or human rendering.

## Owned Surface

The crate owns:

- command tree construction below the process entrypoint;
- command-level argument and configuration resolution;
- orchestration of validate, plan, run, inspect, replay, diff, cache, import,
  export, repair, and diagnostic workflows;
- typed command response and error models;
- JSON envelope selection and human rendering;
- operator-facing refusal and recovery guidance;
- checked-in command reference generation.

It does not own graph semantics, scheduler behavior, backend execution,
artifact serialization, or process startup.

## Internal Boundaries

| Path | Responsibility |
| --- | --- |
| `../src/commands/` | command models, shared orchestration, output contracts, and reference docs |
| `../src/routes/` | route selection, preconditions, command-family execution, and response shaping |
| `../src/graph/` | graph command orchestration |
| `../src/read/` | graph, run, and runtime input loading |
| `../src/replay/` | replay and diff application services |
| `../src/inspect/` | status, doctor, integrity, comparison, and failure views |
| `../src/cache/` | cache command and service orchestration |
| `../src/repair/` | explicit repair proposals and operations |
| `../src/explain/`, `../src/format/`, `../src/migrate/` | focused command families |

Routes remain thin enough to expose preconditions and delegate domain behavior.
Graph, runtime, or persistence algorithms do not belong in route handlers.

## Orchestration Contract

A command workflow:

1. resolves and validates inputs;
2. establishes explicit roots, configuration, and policy;
3. calls the owning domain package;
4. maps the domain result into a typed response;
5. renders exactly one selected output mode;
6. preserves status through the process boundary.

Human and JSON modes must report the same operation outcome. Human output may
be more explanatory, but cannot weaken refusals or omit the causal failure
class. JSON output must remain parseable on failure when the command promises a
JSON envelope.

## Input And Path Contract

Commands must distinguish source files, run roots, run identifiers, artifact
identifiers, and output destinations. Preview and inspection routes do not
mutate retained state. Mutating commands disclose their target and refuse
unsafe path relationships before domain execution.

Configuration precedence and deprecation behavior remain visible. A command
must not silently replace an invalid explicit value with a profile or default.

## Stability

`stable` and `prelude` are the curated Rust integration lanes. The installed
command surface is additionally governed by generated CLI reference and
compatibility fixtures. Experimental routes or helpers do not become stable
because they are present in source.

Command names, arguments, JSON envelopes, exit behavior, and retained output
layout are compatibility-sensitive.

## Dependency Direction

The app depends on:

- `bijux-dag-core` for graph meaning;
- `bijux-dag-runtime` for execution behavior;
- `bijux-dag-artifacts` for retained evidence.

`bijux-dag-cli` depends on the app as a thin process wrapper. The app must not
depend on the CLI, testkit, or maintainer packages.

## Failure Contract

Operator input must not panic the application. Failures distinguish malformed
input, graph rejection, policy refusal, unsupported capability, runtime
failure, evidence corruption, unsafe path, and rendering/internal defects.

Repair, replay, and import commands preserve the original evidence and report
what was proposed or changed. They do not rewrite a failed run into successful
history.

## Verification

| Claim | Required evidence |
| --- | --- |
| package boundary | `crates/bijux-dag-app/tests/crate_boundary_contract.rs` |
| command tree and routing | CLI and command-surface routing contracts |
| machine output | output, error-output, schema lockstep, and snapshot contracts |
| no-panic operator handling | operator input and route-entrypoint no-panic contracts |
| run/replay/import/export | owning workflow and retained-evidence contracts |
| public Rust lane | `crates/bijux-dag-app/tests/public_api_contract.rs` |

Use focused contract files for a bounded route. Broad orchestration changes
require the package suite:

```bash
cargo test --locked -p bijux-dag-app
```

Public workflows are explained under `docs/bijux-dag/`. Cross-crate normative
rules remain under `docs/spec/`; this page does not duplicate them.
