# Security Model

## Purpose
Define the operational security posture and risk model for running bijux-dag workloads.

## Context
Execution systems process commands and artifacts; security is therefore a first-class operational contract.

## Explanation
Security model goals:
- minimize unintended privilege escalation.
- preserve integrity of run and artifact evidence.
- isolate execution context according to backend capabilities.

Threat categories:
- untrusted DAG content or command input.
- compromised execution environment or adapter.
- artifact tampering or evidence manipulation.
- secret leakage through logs, artifacts, or command arguments.

Operational controls:
- run with least privilege credentials.
- isolate runners/agents from sensitive network zones by default.
- use explicit secret injection boundaries and avoid secret persistence in artifacts.
- preserve immutable run evidence for audit and incident analysis.

Incident response expectations:
- classify severity by integrity/availability/confidentiality impact.
- preserve failing run context for forensic analysis.
- rotate credentials and invalidate compromised execution scopes.

## Examples
```text
Control examples:
- deny outbound network by default for local test lanes
- pass tokens via secure CI secret store, not inline command flags
- redact known secret patterns in run logs
```

```text
Security incident triage fields:
- incident_id
- affected_run_ids
- suspected_scope
- mitigation_status
```

## Guarantees
- Security responsibilities and risk classes are explicit.
- Integrity of run/artifact evidence is treated as an operational requirement.
- Secret-handling boundaries are part of normal operating procedure.

## Limitations
- This model does not claim complete protection from host-level compromise.
- Backend-specific isolation strength varies by environment.
- Security model guidance does not replace organization-wide policies.

## Related
- `docs/07-operations/04-trust-boundaries.md`
- `docs/07-operations/05-backend-support.md`
- `docs/06-specification/02-run-model.md`
- `docs/06-specification/03-artifact-model.md`
