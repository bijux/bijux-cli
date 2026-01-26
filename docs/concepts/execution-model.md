# Execution model

## Purpose
This document tells you exactly how a command runs, from argv to exit.

## Scope
It covers the CLI process only. It does not cover the API or plugin internals.

## What problem this solves
When a command fails, you need to know where the decision was made and why.
You also need to know which step is allowed to change output or exit codes.

## Why you should care
If you can trace a failure to one step, you fix bugs fast and avoid regressions.
If you understand the order, you stop accidental side effects from creeping in.

## What confusion this removes
It removes the guesswork about when policy is resolved and when output is chosen.

## Guarantees
Bijux guarantees:
1. Intent is built once and never mutated.
2. Policy is resolved once before runtime initialization.
3. Runtime initialization happens before command dispatch.
4. ExitIntent is the only way the CLI exits.

## How to Think About This
Think of the CLI as a single, linear pipeline.
Each step consumes input and produces a new, immutable artifact.
If a decision is not made in its step, it is forbidden elsewhere.

## Common Misunderstandings
- "Commands can decide output format." They cannot. Output is decided in policy resolution.
- "Runtime can re-read flags." It cannot. Intent and policy are already fixed.

## Execution
1. Parse argv into a CLI intent.
2. Resolve policy for output routing and formatting.
3. Initialize runtime: DI, services, plugins.
4. Dispatch the command.
5. Emit output and exit via ExitIntent.

## Failure Modes
- Invalid flags: exit 2 with structured error.
- Unknown command: exit 2 with structured error.
- Runtime init failure: exit 1.
- Command failure: exit determined by exit policy.

No recovery happens inside the CLI. The process exits deterministically.

## Design Rationale
We deliberately chose a single linear flow because it prevents late overrides.
Why not resolve policy inside each command? That creates drift and hidden side effects.

## Non-Goals
- REPL internals.
- Plugin metadata format.

## References
- Implementation: `src/bijux_cli/core/bootstrap_flow.py`
- Policy resolution: `src/bijux_cli/core/precedence.py`
- Regression coverage: `tests/regression/test_bootstrap_paths.py`
