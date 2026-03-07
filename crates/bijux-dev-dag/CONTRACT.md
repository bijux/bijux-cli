# bijux-dev-dag contract

## Authority

`bijux-dev-dag` is the repository control-plane authority for policy validation,
contract checks, governance suites, and release verification composition.

## Makefile boundary

The Makefile is a convenience wrapper only. It must delegate to
`bijux-dev-dag` commands and must not implement independent governance logic.

## Scope

- dependency and crate-boundary policy enforcement
- source and documentation guardrails
- schema and contract suite orchestration
- release verification suite composition

## Non-scope

- product runtime orchestration behavior
- DAG execution business logic
