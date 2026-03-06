# RBAC and authorization policy contracts

This document defines typed RBAC and policy-evaluation contracts for control-plane authorization.

## Subject, action, and resource model

Subjects:

- user
- service account
- worker
- scheduler
- automation identity

Actions:

- read
- write
- execute
- approve
- manage
- administer
- audit

Resources:

- DAG
- DAG version
- run
- node
- artifact
- schedule
- queue
- policy
- tenant

## Hierarchical resource scopes

- global
- tenant
- DAG
- run

Policy requests always include subject, action, resource, scope, and environment.

## Policy decisions and traces

Decision types:

- allow
- deny
- conditional
- delegated

Each evaluation returns:

- `PolicyDecisionRecord` (decision + policy bundle identity)
- `PolicyEvaluationTrace` (evaluated/matched rules and deterministic reason path)

## Roles and composition

Built-in roles:

- viewer
- operator
- developer
- releaser
- tenant admin
- platform admin
- auditor

Custom roles are validated and rejected for unsupported high-risk permission combinations.

## Environment-aware authorization

Environment rules can deny specific actions in hardened environments (for example production).

## Permission boundaries

Distinct boundaries are modeled for:

- run-control permissions
- DAG publication permissions
- artifact-access permissions

Sensitive controls are independently governed for replay/export/promotion/retention-override.

## Identity-specific permissions

- scheduler identity permissions
- worker identity permissions

## Policy bundle versioning and cache behavior

- policy decisions record bundle id + version for reproducible audits.
- decision cache keys include subject/action/resource/environment/bundle version.
- cache invalidation is strict by policy bundle version.

## Dry-run and acceptance coverage

- dry-run mode reports would-allow/would-deny outcomes before rollout.
- acceptance checks enforce least-privilege and cross-tenant denial guarantees.

Fixtures:

- `crates/bijux-dag-runtime/tests/fixtures/policy/org_models.json`
