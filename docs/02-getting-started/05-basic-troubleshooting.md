# Basic Troubleshooting

## Purpose
Provide first-response diagnostics for common setup and execution failures.

## Context
This is the default guide for early lifecycle failures in installation, authoring, and first execution attempts.

## Explanation
Troubleshoot in this order:
1. Confirm CLI resolution and command discovery.
2. Validate DAG structure.
3. Check dependency ordering and referenced nodes.
4. Check artifact path expectations.
5. Check replay/diff mismatch interpretation.

Common issues and first checks:

CLI not found
- Cause: binary not on `PATH`.
- Check: `command -v bijux-dag`.

DAG validation failure
- Cause: malformed graph shape or missing required fields.
- Check: run with validation-oriented command surface and inspect reported field path.

Missing artifact error
- Cause: node expected output path was never produced.
- Check: inspect failing node command and artifact path assumptions.

Replay mismatch
- Cause: input/environment/runtime divergence.
- Check: compare run context and diff diagnostics for classification.

Unknown command or wrong flags
- Cause: command surface mismatch or typo.
- Check: use CLI help discovery path.

CLI help discovery path:
- `bijux-dag --help`
- `bijux-dag <group> --help`
- `bijux-dag <group> <command> --help`

## Examples
```bash
# Command discovery
bijux-dag --help
bijux-dag run --help
bijux-dag inspect --help

# DAG validation-oriented execution
bijux-dag run --dag ./examples/first.dag.json

# Replay mismatch diagnosis
bijux-dag replay --run-id RUN_20260309_001
bijux-dag diff run --left RUN_20260309_001 --right RUN_20260309_002
```

## Guarantees
- The troubleshooting path prioritizes highest-signal checks first.
- The listed failure categories map to common beginner errors.

## Limitations
- This guide is not a full incident-response playbook.
- Deep backend or distributed failures are handled in operations docs.

## Related
- `docs/02-getting-started/01-installation.md`
- `docs/02-getting-started/03-running-a-pipeline.md`
- `docs/03-user-guide/07-inspect-and-debug.md`
- `docs/07-operations/01-ci-integration.md`
