# Platform operating model

## Roles

- DAG author: defines workflow logic and contract-compliant task surfaces.
- Operator: monitors runs, performs interventions, and executes runbooks.
- Releaser: approves release evidence, compatibility status, and rollback readiness.
- Tenant admin: manages tenant policy overlays, quotas, and access.
- Platform admin: owns global policy, scheduler topology, and cross-tenant guardrails.

## Service objectives

Core SLO targets:
- run creation latency
- dispatch latency
- completion reliability
- artifact availability

SLIs must be tied to durable emitted metrics and audit evidence.

## Error budget and incidents

Error budget policy tracks scheduler outages, backend degradations, and artifact corruption incidents.

Incident severity levels:
- critical
- high
- medium
- low

## Operational preparedness

Runbooks must exist for:
- scheduler failover
- backend outage
- artifact corruption
- policy misconfiguration
- tenant isolation breach

Gameday exercises and postmortem templates are mandatory for high-severity classes.

## Release governance

Promotion to stable requires:
- evidence bundle
- compatibility result set
- rollback plan

## Audit and supportability

Regulated deployments require audit-readiness checklist completion.

Supportability explicitly distinguishes:
- official plugins
- supported backends

## Platform boundaries

Platform guarantees and operator responsibilities are documented separately and must be evaluated before production onboarding.
