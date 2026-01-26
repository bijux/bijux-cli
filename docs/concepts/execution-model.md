# Execution model

## Purpose
This document guarantees how bijux-cli executes a command from argv to exit.

## Scope
This is the CLI execution flow only. It does not describe API usage or plugin internals.

## Core Concepts
- Intent: parsed command name, arguments, and resolved flags.
- Policy: effective routing and formatting rules resolved once.
- Runtime: initialized services, DI container, and plugin registry.
- ExitIntent: structured decision for code and output.

## Invariants
- Intent is built once and never mutated.
- Policy is resolved once before runtime initialization.
- Runtime initialization happens before command dispatch.
- ExitIntent is the only way to exit the CLI.

## Execution
1. Argument parsing builds a CLI intent.
2. Policy resolution computes output routing and formats.
3. Runtime initializes DI, services, and plugins.
4. Command dispatch executes the intent.
5. ExitIntent emission writes output and exits.

## Failure Modes
- Invalid flags: return exit code 2 with structured error.
- Unknown command: return exit code 2 with structured error.
- Runtime init failure: return exit code 1.
- Command failure: return exit code based on exit policy.

No recovery occurs inside the CLI. The process exits deterministically.

## Design Rationale
- Alternatives: late policy resolution in commands.
- Rejected because it creates inconsistent behavior and hidden side effects.
- Chosen: single policy resolution to guarantee determinism.

## Non-Goals
- Interactive REPL lifecycle details.
- Plugin metadata format.

## References
- Implementation: `src/bijux_cli/core/bootstrap_flow.py`
- Policy resolution: `src/bijux_cli/core/precedence.py`
- Regression coverage: `tests/regression/test_bootstrap_paths.py`
