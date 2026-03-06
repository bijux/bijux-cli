# AI-assisted debugging, planning, and operator workflows

## Evidence-first design

AI-assisted workflows operate on structured investigation bundles and machine-readable failure summaries.

Primary summaries include:
- failed/stuck run signals
- policy denials
- artifact mismatch diagnostics
- schedule and planner anomalies

## Diagnostics API expectations

Diagnostics answers must include explicit evidence references, not only free-text conclusions.

## Safe recommendation boundary

Recommended actions are restricted to safe operator operations:
- replay
- verify
- inspect lineage
- suppress schedule

Recommendations must satisfy policy and permission guardrails.

## Change analysis and similarity

The system supports:
- what-changed summaries against prior successful runs
- incident similarity lookup for known failure patterns
- replay minimization recommendations

## Operator review loop

Operators can accept, reject, or annotate suggestions.

Automated suggestions never execute autonomously.

## Privacy and redaction

Any exported evidence for automated analysis must apply secret, PII, and tenant-sensitive metadata redaction policies.

## Simulation and maturity

Recommendation quality is evaluated through historical simulation scenarios.

Maturity progression:
- diagnostics only
- evidence-guided suggestions
- guarded recommendations
