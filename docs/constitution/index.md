# Constitution

## Purpose
This section defines durable command-line contracts for `bijux-cli`.

## Scope
It covers public behavior that must remain stable across implementation changes.

## Core Concepts
- Contracted behavior is binding for users and automation.
- Incidental behavior can change without compatibility guarantees.

## Invariants
- `bijux` remains the canonical binary interface.
- Changes that break a documented contract require an explicit major-version policy decision.

## Documents
- [CLI Constitution](CLI_CONSTITUTION.md)
- [Compatibility Promise](COMPATIBILITY_PROMISE.md)
- [Global Flags](GLOBAL_FLAGS.md)
- [Exit Codes](EXIT_CODES.md)
- [Error Envelope](ERROR_ENVELOPE.md)
- [Output Envelope](OUTPUT_ENVELOPE.md)
- [Stdout/Stderr Rules](STDOUT_STDERR_RULES.md)
- [REPL Parity](REPL_PARITY.md)
- [Plugin Namespace Policy](PLUGIN_NAMESPACE_POLICY.md)
- [Plugin Lifecycle](PLUGIN_LIFECYCLE.md)
- [Plugin Sandboxing Policy](PLUGIN_SANDBOXING_POLICY.md)
- [Python Distribution Policy](PYTHON_DISTRIBUTION_POLICY.md)
- [Deprecation Policy](DEPRECATION_POLICY.md)
