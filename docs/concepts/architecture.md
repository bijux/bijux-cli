# Architecture

Bijux CLI is built around a small set of core components:

- CLI entry: parses arguments and builds a CLI intent
- Policy resolution: computes effective flags and output rules
- Runtime: initializes DI, plugins, and command dispatch
- Emission: outputs payloads according to the resolved policy
- Services: config, history, diagnostics, and plugin registry

The execution path is linear: intent -> policy -> runtime -> dispatch -> emit.
There are no hidden fallbacks or IO during parsing.

Key design goals:

- Strict separation of parsing vs execution
- Explicit output routing (stdout/stderr) in core
- Predictable exit codes and errors
- Plugins are validated and activated explicitly
