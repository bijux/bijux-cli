# Inspect Commands

Use `inspect` commands to retrieve authoritative run and artifact evidence before replay or diff decisions.

## Inspect targets and expected use

- `inspect run`: execution status, first failing node, node-level outcomes.
- `inspect artifact`: identity, lineage, and artifact evidence fields.

Diagnostic rule: inspect first, replay second, diff third.

## Core invocation patterns

```bash
bijux-dag inspect --help
bijux-dag inspect run --run-id RUN_20260309_220 --output json
bijux-dag inspect artifact --artifact-id ART_20260309_902 --output json
```

## Output modes and common failure states

Output modes:

- human mode for interactive triage,
- JSON mode for automation and incident records.

Failure states:

- unknown run/artifact selector,
- malformed selector input,
- evidence retrieval failure due to missing/corrupt storage records.

## Multi-command diagnostic walkthrough

```bash
bijux-dag inspect run --run-id RUN_20260309_220 --output json
bijux-dag inspect artifact --run-id RUN_20260309_220 --output json
bijux-dag replay --run-id RUN_20260309_220
bijux-dag diff run --left RUN_20260309_204 --right RUN_20260309_220
```

This sequence keeps diagnosis evidence-driven instead of guess-driven.

## Next reading

- Run command reference and ID selection: [Run Commands](../04-cli-reference/03-run-commands.md)
- Debug strategy and interpretation: [Inspect And Debug](../03-user-guide/07-inspect-and-debug.md)
