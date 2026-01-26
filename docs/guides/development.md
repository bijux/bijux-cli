# Development

## Purpose
This guide provides practical guidance for contributors working on bijux-cli. It exists to reduce accidental violations of core invariants and to help new contributors make changes safely.

## Scope
It covers where to add commands, how to test, and how to avoid breaking core guarantees. It does not cover project governance or release processes.

## Where to Add Commands
Add new CLI commands under `src/bijux_cli/cli/commands/` and keep command handlers thin. Core logic should live in services so it can be tested independently of CLI wiring.

## Where Not to Add Logic
Do not add policy decisions or output routing in infra modules. Those decisions must remain in core so tests can enforce invariants consistently.

## Testing Expectations
Run unit and regression tests for the areas you touched, and confirm that core invariants remain intact. If you change behavior, add or update tests that assert the new guarantees.

## Tooling Notes
The project enforces linting, quality checks, and security scans. These checks are not optional; they preserve the guarantees documented in the concepts section.

## References
- [Contributor mental model](contributor-mental-model.md)
- [Architecture walk-through](../architecture/walkthrough.md)
