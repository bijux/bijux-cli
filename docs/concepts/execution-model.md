# Execution Model

## Purpose
This document defines the execution model of bijux-cli. It states where decisions are made, where they are forbidden, and what invariants are preserved across every run.

## Scope
It covers the CLI execution path from argument parsing through exit. It does not cover internal APIs beyond what is needed to explain the model.

## Core Concepts
The CLI is a deterministic pipeline: argv becomes a CLI intent, intent becomes a resolved policy, policy gates runtime initialization, commands execute within that runtime, and output is emitted once using a single routing decision.

## How to Think About This
Think of each stage as a contract boundary. Parsing must be pure, policy resolution must be definitive, runtime initialization must be late and deliberate, and emission must be final and unambiguous. If you are debugging behavior, identify which boundary the behavior should belong to and verify it does not leak across boundaries.

## Invariants
- Policy is resolved exactly once and never mutated later.
- Runtime initialization does not occur on fast paths like `--help` or `--version`.
- Output routing is decided once in core and enforced uniformly.

## Execution Flow
Execution always follows this sequence: parse arguments, build an intent, resolve policy, initialize runtime services, dispatch the command, and emit output. Fast paths exit early after intent resolution, without initializing runtime services.

## Failure Modes
Parsing errors return a user-facing error and a stable exit code. Policy resolution failures return a structured error and prevent runtime initialization. Dispatch failures emit an error payload and exit according to the exit policy.

## Design Rationale
This model prevents subtle ordering bugs and guarantees consistent behavior across different environments. By limiting where decisions can happen, it keeps the system predictable and testable.

## Non-Goals
This document does not describe how commands are implemented. It only defines the required execution boundaries.

## References
- Implementation: `src/bijux_cli/core/bootstrap_flow.py`
- Policy resolution: `src/bijux_cli/core/precedence.py`
- Regression coverage: `tests/regression/test_bootstrap_paths.py`
