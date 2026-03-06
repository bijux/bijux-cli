# Authentication, identity, and trust bootstrap contracts

This document defines typed identity/authentication contracts and trust bootstrap rules for schedulers, workers, CLI clients, and service clients.

## Authentication boundary

Supported boundary abstraction:

- local development credentials
- service tokens
- future OIDC flow contract

Identity model is transport-agnostic and typed via `IdentityPrincipal`.

## Credential scopes and lifecycle

Credential scopes:

- CLI
- API client
- scheduler
- worker

Credential lifecycle supports issue time, expiry, renewal policy, and max renewals.

Short-lived worker credentials are modeled via lease/run-scoped bindings.

## Provenance, revocation, and audit

- mutating actions record credential provenance (principal, credential class, timestamp).
- revocation records include reason and propagation policy for running operations.
- authentication events are audited for login, refresh, revoke, and failure.

## Trust bootstrap flows

- worker enrollment trust flow
- scheduler replica bootstrap trust flow
- plugin trust registration requiring explicit approver and approval ticket

## Artifact signing identity and trust domains

- artifact signing identity is first-class and attributed to trust domain.
- trust domains include tenant, environment, and execution backend dimensions.

## Mutual-auth and local bypass rules

- mutual-auth design notes are explicit for worker/control-plane links.
- local development bypass rules are isolated by environment and explicit markers.

## Provider migration compatibility

Identity provider migration is compatible only when subject identity and audit chain are preserved.

## Conformance fixtures

- `crates/bijux-dag-runtime/tests/fixtures/auth/expired_credentials.json`
- `crates/bijux-dag-runtime/tests/fixtures/auth/revoked_credentials.json`
- `crates/bijux-dag-runtime/tests/fixtures/auth/wrong_tenant_credentials.json`
- `crates/bijux-dag-runtime/tests/fixtures/auth/downgraded_scopes.json`

## Trust health command

`bijux-dev-dag` exposes a trust health binary command:

- `cargo run -p bijux-dev-dag --bin trust_health`

Output includes active identities, credential classes, and policy baselines.

## Federation readiness criteria

Readiness requires:

- local auth bypass isolated to local environment
- revocation propagation support
- short-lived worker credential support
- complete authentication audit event coverage
