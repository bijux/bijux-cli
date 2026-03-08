# Backend policy overlays

Backend policy overlays provide backend-specific operational settings without leaking backend semantics into the core DAG specification.

## Overlay intent

- Keep DAG core contracts backend-agnostic.
- Apply execution restrictions at deployment/runtime policy layers.
- Preserve replay and validation determinism across backends.

## Overlay examples

- kubernetes image allowlist policy
- hpc queue/account constraints
- external service endpoint restrictions
- container runtime syscall profile settings

## Governance rules

- Overlay evaluation must happen after graph compile and before dispatch.
- Overlay decisions must be logged in run/audit artifacts.
- Overlay policy identifiers and versions must be included in execution provenance.
