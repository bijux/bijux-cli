# Security Model

This document states the bijux-dag security posture: least-privilege execution, explicit trust boundaries, and integrity-preserving evidence.

## Threat model assumptions

Assume attackers may:
- supply malicious DAG or bundle inputs,
- tamper with artifacts or run records after execution,
- exploit weak backend isolation,
- exfiltrate secrets through logs, command arguments, or artifacts.

Assume operators control:
- runtime deployment and credential policy,
- CI policy gates,
- storage and retention policy for run/artifact evidence.

## Security posture and non-guarantees

Posture commitments:
- untrusted inputs are validated before execution,
- privileged actions are explicit and auditable,
- run/artifact evidence integrity is operationally protected,
- secret material is handled through bounded injection paths.

Explicit non-guarantees:
- no protection claim against fully compromised host root,
- no safety claim for unverified imported bundles,
- no confidentiality guarantee when operators bypass secret-handling controls.

## Policy vs enforcement vs operator responsibility

Policy defines required behavior:
- least privilege,
- input verification,
- evidence retention,
- secret handling and rotation rules.

Enforcement is what tooling checks:
- CI gate rules,
- runtime validation/verification checks,
- access controls and audit trails.

Operator responsibility is what people must do correctly:
- choose trusted execution environments,
- deny unsafe overrides,
- rotate credentials after suspected compromise,
- treat unresolved trust state as blocking.

## Trust boundary violations in practice

Treat these as trust boundary violations:
- executing external bundle content before integrity/provenance verification,
- promoting releases when replay/diff evidence is missing or `unknown`,
- accepting artifact lineage with missing producing run/node links,
- running high-privilege workloads in shared or unisolated environments.

Each violation requires re-verification and incident logging before promotion resumes.

## Operational controls

Recommended baseline controls:
- default-deny network egress for low-trust lanes,
- short-lived credentials only,
- secret redaction for retained logs,
- immutable evidence storage for release-critical runs.

## Guarantees

- Security claims are bounded and testable.
- Responsibilities are separated so accountability is clear.
- Trust-boundary violations are actionable events, not warnings.

## Non-guarantees

- This model does not replace organization-wide compliance policy.
- Backend isolation strength is bounded by backend capability.

## Next reading

- [Trust boundaries](04-trust-boundaries.md)
- [Run model contract](../06-specification/02-run-model.md)
- [Artifact model contract](../06-specification/03-artifact-model.md)
