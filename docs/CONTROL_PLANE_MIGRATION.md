# Control plane migration boundary

## Server crate placeholder

Planned crate name: `bijux-dag-api`.

Purpose:

- host service control-plane endpoints
- integrate persistent registry and scheduler backends
- enforce authorization and policy decisions

## What remains in CLI now

- repository-oriented quality checks
- local compatibility and contract execution
- local schedule validation and preview
- local artifact and observability reporting

## What migrates to service control plane later

- shared DAG registry publication workflow
- remote schedule persistence and trigger evaluation
- multi-user run-control operations
- policy bundle distribution and decision endpoints
- organization-scoped authorization decisions

## Compatibility intent

`bijux-dev-dag` keeps stable command semantics while transport and persistence move behind `bijux-dag-api`.
