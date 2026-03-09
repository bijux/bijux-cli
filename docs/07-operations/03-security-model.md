# Security Model

Define the operational security posture and risk model for running bijux-dag workloads.

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

Security recommendations:
- enable command allow-lists for high-trust environments where feasible.
- treat imported bundles as untrusted until integrity and provenance checks pass.
- redact secret-like tokens in logs before evidence retention.
- enforce credential rotation on every confirmed execution-scope compromise.

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
- Includes practical control recommendations for day-to-day operations.

## Limitations
- This model does not claim complete protection from host-level compromise.
- Backend-specific isolation strength varies by environment.
- Security model guidance does not replace organization-wide policies.

## Related
- `docs/07-operations/04-trust-boundaries.md`
- `docs/07-operations/05-backend-support.md`
- `docs/06-specification/02-run-model.md`
- `docs/06-specification/03-artifact-model.md`

## Security posture statement

bijux-dag security posture is evidence-preserving, least-privilege execution with explicit trust boundaries:

- assume DAG inputs and imported bundles are untrusted until validated,
- limit execution permissions to minimum required scope,
- preserve run/artifact evidence integrity for audit and incident response,
- separate policy decisions from runtime mechanics so security claims stay testable.

This posture favors verifiable controls over broad security marketing claims.

## Threat model, trust assumptions, and explicit non-guarantees

Threat model assumptions:

- adversary may control DAG input content,
- adversary may attempt artifact/log tampering post-execution,
- adversary may exploit weakly isolated execution backends.

Trust assumptions:

- trusted root of control: repository policy + reviewed runtime binaries,
- conditional trust: backend environment only within declared support envelope,
- untrusted by default: imported bundles and external DAG inputs until verified.

Explicit non-guarantees:

- no guarantee against fully privileged host compromise,
- no guarantee that unverified external bundles are safe to execute,
- no guarantee of confidentiality if operators bypass secret-handling boundaries.

## Policy, enforcement, and operator responsibility

- policy: defines required controls (least privilege, verification, retention).
- enforcement: runtime/CI mechanisms that implement policy checks.
- operator responsibility: selecting trusted environments, managing secrets, and refusing unsafe overrides.

Security posture fails when policy exists but enforcement is bypassed or operator actions violate trust boundaries.
