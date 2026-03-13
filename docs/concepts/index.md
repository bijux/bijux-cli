# Concepts

## Purpose
This section defines the guarantees and mental models that make bijux-cli predictable. It exists to explain the rules the CLI must follow so you can reason about behavior without reading source code.

## Scope
Concepts documents the execution model, precedence rules, exit behavior, logging semantics, and plugin lifecycle. It does not provide step-by-step usage instructions or command reference tables.

## Audience
Engineers who want to understand why the CLI behaves the way it does should start here. This section is designed to eliminate ambiguity and to provide stable guarantees that tests enforce.

## Architecture

Bijux-cli follows a linear pipeline:

1. argument parsing builds an intent
2. policy resolution computes the immutable runtime configuration
3. runtime services initialize only after that policy is fixed
4. command execution runs inside that runtime
5. output is emitted exactly once

This keeps decision ownership clear and makes late overrides easier to reject.
Parsing is data construction, policy resolution is the single source of truth,
runtime initialization is a controlled side effect, and emission is the final
step.

## Execution Model

Each run follows the same contract boundary:

- parsing must stay pure
- policy is resolved once and never mutated later
- fast paths such as `--help` and `--version` must not initialize the runtime
- output routing is decided once and enforced uniformly

When one of those boundaries leaks, the usual symptoms are nondeterministic
output, broken CLI and API parity, or inconsistent exit behavior.

## Precedence

Configuration resolution is a strict merge, not a negotiation:

1. CLI flags override environment variables and config files
2. environment variables override config files
3. defaults apply only when no other source provides a value

If a source is invalid, resolution fails instead of partially applying
overrides.

## Exit Policy

Exit codes are part of the public automation contract.

- `0` success
- `1` internal or general failure
- `2` usage or user-input failure
- `3` encoding or serialization failure
- `130` user abort or signal interruption

Formatting and presentation flags must not change exit codes.

## Logging

Logs are diagnostic output, not command results.

- output format flags change command payload rendering, not log policy
- log level changes verbosity, not command semantics
- logging failures must not suppress command output or rewrite exit codes

For automation, structured stdout and the exit code remain the stable signals.

## Plugin Lifecycle

Plugins move through explicit lifecycle states:

- discovered
- installed
- active
- inactive
- removed

Transitions are expected to validate metadata first, keep registry and
filesystem state aligned, and roll back partial state on failure rather than
leaving ambiguous installs behind.

## Related Pages

- [Architecture decision rules](../architecture/decision-rules.md)
- [Exit codes](../reference/exit-codes.md)
- [Environment and precedence inputs](../reference/environment.md)
- [Plugin state](../plugin_state.md)
