# Basic Troubleshooting

## Purpose
Provide a minimal, high-signal troubleshooting path for setup and first-run failures.

## Context
This guide is for common failures encountered during first project execution.

## Explanation
Troubleshooting sequence:
1. command discovery works
2. DAG file validates
3. dependency IDs are correct
4. expected artifact paths exist
5. replay/diff mismatch is classified

Common issues:
- CLI command not found
- invalid DAG shape
- missing dependency node
- missing artifact output
- replay mismatch due to context drift

CLI help discovery:
- `bijux-dag --help`
- `bijux-dag run --help`
- `bijux-dag inspect --help`

Quality checks completed for getting-started flow:
- tutorial command sequence is coherent
- examples are simple and minimal
- command/output expectations are aligned to documented steps

## Examples
```bash
command -v bijux-dag
bijux-dag --help
bijux-dag run --dag ./examples/first.dag.json
bijux-dag inspect run --run-id RUN_20260309_001
bijux-dag replay --run-id RUN_20260309_001
```

## Guarantees
- This sequence prioritizes highest-signal checks first.
- Beginner troubleshooting scope is explicit and practical.

## Limitations
- This guide is not a full incident response playbook.
- Complex distributed failures are handled in operations docs.

## Related
- `docs/02-getting-started/01-installation.md`
- `docs/02-getting-started/03-running-a-pipeline.md`
- `docs/03-user-guide/07-inspect-and-debug.md`
- `docs/07-operations/01-ci-integration.md`
