---
title: Compliance-Gated Bulletin Workflow
audience: operators
type: operations
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-07
---

# Compliance-Gated Bulletin Workflow

This guide demonstrates a real failure-recovery workflow built around
`evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json`.

The graph prepares a bulletin from a source note, retries a transient
compliance lookup, fails cleanly when the publication gate is not approved, and
repairs the run with a focused replay boundary after the approval file is
corrected.

## What This Workflow Proves

- retry evidence is retained when a transient node succeeds on a later attempt
- `max_attempts = 2` means two retries after the initial attempt, so an
  exhausted run records three total attempts
- the root failure stays attached to the approval gate instead of being blurred
  into a generic downstream failure
- `replay --from-node validate_publication_gate` reuses verified upstream
  artifacts from the source run and reruns only the failed tail
- the repaired run can be verified strictly before promotion

## Prepare The Run

Run these commands from repository root:

```bash
GRAPH_PATH="evidence/dag/authoring/examples/compliance-gated-bulletin.dag.json"
SOURCE_NOTE="$(pwd)/evidence/dag/authoring/examples/compliance-gated-source/team-update.md"
RUN_ROOT="./artifacts/compliance-gated-runs"
RETRY_PLAN="./artifacts/compliance-gated-retry-plan.json"
PUBLICATION_GATE="./artifacts/compliance-gated-publication-gate.json"
```

## Validate The Graph

```bash
bijux-dag validate "${GRAPH_PATH}"
```

## Scenario One: Retry Succeeds, Approval Fails

Write a retry plan that forces one transient failure and a publication gate that
is still not approved:

```bash
cat > "${RETRY_PLAN}" <<'EOF'
{
  "fail_until_attempt": 1,
  "gate_policy": "manual-approval",
  "expected_reviewer_group": "release-managers"
}
EOF

cat > "${PUBLICATION_GATE}" <<'EOF'
{
  "approved": false,
  "reviewer": "",
  "reviewer_group": "release-managers"
}
EOF
```

Run the workflow:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id compliance-gated-source \
  --input "source_note=${SOURCE_NOTE}" \
  --input "retry_plan=${RETRY_PLAN}" \
  --input "publication_gate=${PUBLICATION_GATE}" \
  --input "bulletin_title=Compliance Review Bulletin"
```

Inspect the retained retry evidence and the causal failure:

```bash
cat "${RUN_ROOT}/run-compliance-gated-source/nodes/fetch_compliance_gate/attempts.json"
cat "${RUN_ROOT}/run-compliance-gated-source/nodes/validate_publication_gate/stderr.log"
bijux-dag --json runs explain-failure compliance-gated-source --root "${RUN_ROOT}"
```

At this point the evidence should show:

- `fetch_compliance_gate` failed once, then succeeded on attempt `2`
- `validate_publication_gate` is the root failure
- `publish_bulletin` is a propagated skip, not an independent fault

## Repair The Approval Boundary

Correct only the approval file:

```bash
cat > "${PUBLICATION_GATE}" <<'EOF'
{
  "approved": true,
  "reviewer": "A. Reviewer",
  "reviewer_group": "release-managers"
}
EOF
```

Replay only the failed tail:

```bash
bijux-dag replay --json \
  --source-run-id compliance-gated-source \
  --source-run-root "${RUN_ROOT}" \
  --out "${RUN_ROOT}" \
  --run-id compliance-gated-repaired \
  --from-node validate_publication_gate
```

Inspect the repaired artifact and verify the run strictly:

```bash
cat "${RUN_ROOT}/run-compliance-gated-repaired/nodes/publish_bulletin/outputs/publish/bulletin.md"
bijux-dag verify --json "${RUN_ROOT}/run-compliance-gated-repaired" --strict
```

The repaired bulletin should now include the reviewer and the retained gate
lookup attempt count.

## Scenario Two: Retry Budget Is Exhausted

Write a retry plan that never recovers inside the configured budget:

```bash
cat > "${RETRY_PLAN}" <<'EOF'
{
  "fail_until_attempt": 9,
  "gate_policy": "manual-approval",
  "expected_reviewer_group": "release-managers"
}
EOF
```

Run the same graph again:

```bash
bijux-dag run --json "${GRAPH_PATH}" \
  --out "${RUN_ROOT}" \
  --run-id compliance-gated-exhausted \
  --input "source_note=${SOURCE_NOTE}" \
  --input "retry_plan=${RETRY_PLAN}" \
  --input "publication_gate=${PUBLICATION_GATE}"
```

Inspect the retry ledger:

```bash
cat "${RUN_ROOT}/run-compliance-gated-exhausted/nodes/fetch_compliance_gate/attempts.json"
cat "${RUN_ROOT}/run-compliance-gated-exhausted/nodes/fetch_compliance_gate/stderr.log"
bijux-dag --json runs explain-failure compliance-gated-exhausted --root "${RUN_ROOT}"
```

The exhausted run should show three retained attempts, with the final
`retry_decision.reason` set to `retry_budget_exhausted`.

## Reading Rule

Use this guide when the question is not whether replay exists in principle, but
whether a real failed run can be diagnosed, repaired at one node boundary, and
verified without rebuilding the entire graph.

## Next Reads

- [Failure Recovery](../failure-recovery.md)
- [Common Workflows](../common-workflows.md)
- [Operator Workflows](../../interfaces/operator-workflows.md)
- [Branching Bulletin Workflow](branching-bulletin-workflow.md)
