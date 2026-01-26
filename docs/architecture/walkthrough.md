# Architecture Walk-through

## Purpose
This document provides a narrative walk-through of what happens when you run `bijux ...`. It exists to make the execution path concrete and to show where each major decision happens.

## Scope
It describes the CLI execution flow from invocation to exit. It does not cover API usage or plugin implementation details.

## Walk-through
When you run the CLI, we first capture the raw argv and construct a CLI intent. The intent is a pure data object that records the command, arguments, and flags without triggering side effects. Next, we resolve policy from CLI flags, environment variables, and configuration files. This is the single point where precedence is applied. After policy resolution, we initialize runtime services such as DI, plugins, and history. Only then do we dispatch the command. The command runs under the resolved policy, and output is emitted exactly once according to the routing decision made in core. Finally, we exit with a code defined by the exit policy.

## Invariants
The walk-through highlights invariants: policy is resolved once, runtime initialization never occurs on fast paths like `--help`, and output routing is not recomputed later.

## Failure Modes
If parsing fails, we return a structured error and exit without initializing runtime services. If policy resolution fails, we emit a stable error payload and exit. If command execution fails, we emit an error and exit using the defined exit policy.

## Design Rationale
A linear execution path avoids hidden branching and makes behavior auditable. This is essential for predictable automation and consistent error handling.

## Non-Goals
This document is not a reference for CLI syntax or configuration keys.
