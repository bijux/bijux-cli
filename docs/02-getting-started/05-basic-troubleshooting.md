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

Detailed failure-first checklist:
1. confirm CLI availability and subcommand help.
2. validate DAG syntax and dependency references.
3. execute run and capture run ID and terminal status.
4. inspect failed node diagnostics.
5. inspect artifact references and file existence.
6. replay baseline when reproducibility is in question.
7. diff baseline vs candidate to isolate scope of drift.
8. classify issue as config error, runtime error, data drift, or environment drift.

Common issues:
- CLI command not found
- invalid DAG shape
- missing dependency node
- missing artifact output
- replay mismatch due to context drift
- scheduler cannot advance because dependencies failed
- CLI argument misuse (wrong flag or missing required value)
- environment mismatch (different shell/tool versions or missing binaries)

CLI help discovery:
- `bijux-dag --help`
- `bijux-dag run --help`
- `bijux-dag inspect --help`

Common failure stories and fixes:
- Invalid DAG:
  - symptom: run command fails before execution.
  - cause: malformed JSON, cycle, or unknown dependency target.
  - action: fix shape/dependencies and re-run validation.
- Missing artifact:
  - symptom: inspect artifact shows missing expected output.
  - cause: producing node failed or wrote to unexpected path.
  - action: inspect node outcome and command output path.
- Replay mismatch:
  - symptom: replay classification reports drift.
  - cause: graph change, input change, environment/tooling drift, or backend capability gap.
  - action: use diff classification and inspect changed scope.
- Scheduler blocked:
  - symptom: downstream nodes never execute.
  - cause: upstream dependency failure.
  - action: debug earliest failed prerequisite node first.
- CLI usage mistake:
  - symptom: unknown flag/subcommand error.
  - cause: command syntax mismatch.
  - action: use contextual `--help` and correct invocation.

## Examples
```bash
# 1) Command presence and help
command -v bijux-dag
bijux-dag --help

# 2) Execute and capture run behavior
bijux-dag run --dag ./examples/first.dag.json

# 3) Inspect run and artifacts
bijux-dag inspect run --run-id RUN_20260309_001
bijux-dag inspect artifact --run-id RUN_20260309_001

# 4) Replay and diff diagnostics
bijux-dag replay --run-id RUN_20260309_001
bijux-dag diff run --left RUN_20260309_001 --right RUN_20260309_002
```

```text
Invalid DAG example:
- node "transform" depends_on: ["prepare", "missing_node"]
Result:
- validation fails because "missing_node" is undefined
```

```text
Environment mismatch example:
- baseline run used tool version X
- candidate run used tool version Y
Result:
- replay/diff may classify drift even if graph is unchanged
```

```text
Quick troubleshooting checklist:
[ ] CLI resolves and help loads
[ ] DAG validates and dependencies are real
[ ] run reached terminal state
[ ] artifact references match expectations
[ ] replay/diff classification reviewed
[ ] environment/tooling parity checked
```

## Guarantees
- This sequence prioritizes highest-signal checks first.
- Beginner troubleshooting scope is explicit and practical.
- Common first-run failure classes include concrete diagnosis and action paths.

## Limitations
- This guide is not a full incident response playbook.
- Complex distributed failures are handled in operations docs.
- Backend-specific low-level diagnostics may require adapter or infrastructure tooling.

## Related
- `docs/02-getting-started/01-installation.md`
- `docs/02-getting-started/03-running-a-pipeline.md`
- `docs/03-user-guide/07-inspect-and-debug.md`
- `docs/07-operations/01-ci-integration.md`
