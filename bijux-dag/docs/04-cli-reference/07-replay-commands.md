# Replay Commands

Use `replay` commands to validate reproducibility and classify equivalence, drift, or incomplete outcomes.

## Replay modes and intent

- normal replay: execute replay and classify against baseline,
- dry-run replay (if supported): validate prerequisites without full execution,
- proof-oriented replay: generate stronger evidence package for release/audit workflows.

Always confirm mode availability with `bijux-dag replay --help` for your build.

## Core invocation patterns

```bash
bijux-dag replay --help
bijux-dag replay --run-id RUN_20260309_220 --output json
bijux-dag replay --run-id RUN_20260309_220 --dag ./pipelines/main.dag.json --output json
```

## Failure, incomplete, and downgrade handling

- failure: replay could not execute required workflow.
- incomplete: replay executed partially but required comparison evidence missing.
- downgrade: replay completed with reduced fidelity due to capability limits.

Downgrade/incomplete are not strict success states; treat them as bounded evidence requiring explicit acceptance.

## Next reading

- Pairing replay with scoped comparisons: [Diff Commands](../04-cli-reference/06-diff-commands.md)
- Formal replay semantics: [Replay Semantics Specification](../06-specification/07-replay-semantics.md)
