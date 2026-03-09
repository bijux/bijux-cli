# Cli Overview

Use the CLI by job-to-be-done, not by memorizing command families.

## Job map

- define and validate workflows: `dag`
- execute and track runs: `run`
- inspect evidence and outputs: `inspect`, `artifact`
- validate reproducibility and change: `replay`, `diff`
- transfer execution context: `bundle`

## Main workflow command map

```text
define -> run -> inspect -> replay -> diff -> bundle (when cross-environment transfer is needed)
```

Typical command sequence:

```bash
bijux-dag dag validate --dag ./pipelines/main.dag.json
bijux-dag run --dag ./pipelines/main.dag.json
bijux-dag inspect run --run-id RUN_20260309_301
bijux-dag replay --run-id RUN_20260309_301
bijux-dag diff run --left RUN_20260309_301 --right RUN_20260309_322
```

## Stable versus advanced surfaces

Stable surfaces (day-to-day operators):

- basic validation,
- run execution/history,
- run/artifact inspect,
- replay and diff for release confidence.

Advanced surfaces (deep diagnostics/automation):

- machine-output integrations,
- capability-sensitive replay/diff flows,
- bundle integrity and portability verification modes.

Start with stable surfaces; adopt advanced surfaces when your workflows require stronger automation or deeper forensics.

## Next reading

- Graph command reference: [Dag Commands](../04-cli-reference/02-dag-commands.md)
- Run and artifact references: [Run Commands](../04-cli-reference/03-run-commands.md), [Artifact Commands](../04-cli-reference/04-artifact-commands.md)
- Validation and comparison references: [Replay Commands](../04-cli-reference/07-replay-commands.md), [Diff Commands](../04-cli-reference/06-diff-commands.md)
