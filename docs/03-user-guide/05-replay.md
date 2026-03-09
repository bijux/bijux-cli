# Replay

## Purpose
Explain how to use replay to validate stability, detect drift, and verify fixes.

## Context
Replay is a core workflow for confidence, especially after dependency or environment changes.

## Explanation
Replay re-executes or re-validates run behavior against a known run context.

Replay planning guidance:
- choose a baseline run with trusted outcome
- choose a candidate run or environment to validate
- define what must remain equivalent and what may differ
- identify required inputs/artifacts before execution starts
- decide strict vs bounded-equivalence expectation for the check

Replay validation guidance:
- compare terminal status
- compare key output/artifact expectations
- compare diagnostics and failure classes when mismatch occurs

Replay guarantees (operator interpretation):
- replay provides explicit outcome classification rather than implicit "looks fine".
- replay can be used repeatedly as a regression confidence tool.
- replay evidence is suitable input for follow-up diff diagnostics.

Replay limitations:
- replay equivalence is bounded by backend capability envelope.
- environment/tooling drift can produce classified divergence even with unchanged graph.
- replay does not guarantee identical timing/resource profile across environments.

When to replay:
- post-upgrade verification
- flaky behavior investigation
- "fixed bug" confirmation

## Examples
```bash
# Replay a known run context
bijux-dag replay --run-id RUN_20260309_010

# Replay with explicit DAG path when needed
bijux-dag replay --run-id RUN_20260309_010 --dag ./pipelines/main.dag.json
```

```text
Replay review checklist:
- status parity checked
- critical artifact expectations checked
- mismatch classified before remediation
```

```text
Replay mismatch example:
- baseline: RUN_010 (succeeded on toolchain X)
- replay: RUN_111 (failed on toolchain Y)
- classification: drift
- next action: run diff and inspect toolchain/environment delta
```

## Guarantees
- Replay is documented as a normal operational tool.
- Validation steps are explicit and repeatable.
- Guarantees and limitations are separated for planning clarity.

## Limitations
- Replay equivalence can still be bounded by environment/backend differences.
- This guide does not define replay algorithm internals.

## Related
- `docs/03-user-guide/04-run-history.md`
- `docs/03-user-guide/06-diff.md`
- `docs/02-getting-started/03-running-a-pipeline.md`
- `docs/06-specification/07-replay-semantics.md`
