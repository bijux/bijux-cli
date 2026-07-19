---
title: Error Codes
audience: mixed
type: interface
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-10
---

# Error Codes

`bijux-dag` exposes registry-governed error identifiers so automation can
classify failures without parsing human messages. This page is the
operator-facing registry; [Error Contract](../../spec/ERROR_CONTRACT.md)
governs how the registry, output envelope, exit behavior, and tests evolve
together.

## Automation Contract

In JSON mode, a command failure returns a non-zero process status and an error
object with stable classification fields:

```json
{
  "ok": false,
  "error": {
    "category": "validation",
    "code": "BJX-VALIDATION-001",
    "exit_code": 3
  }
}
```

Callers may branch on `error.code`, use `error.category` for broader routing,
and retain `error.exit_code` with the command evidence. Human-facing messages
may become clearer without constituting an API change; message text is not a
stable parser surface.

Process exit codes and public error identifiers solve different problems:

| Exit | Proven meaning |
| --- | --- |
| `0` | command completed successfully |
| `2` | parse-class input failure, including malformed DAG JSON |
| `3` | validation or input-access failure recognized by the command contract |
| other non-zero | command failure; inspect the structured category and code |

Do not infer a specific public identifier from the process status alone.

## Public Registry

| Code | Category | Owner | Meaning |
| --- | --- | --- | --- |
| `BJX-PARSE-001` | `parse` | `bijux-dag-core` | Input is not valid DAG JSON |
| `BJX-SCHEMA-001` | `schema` | `bijux-dag-core` | JSON shape violates schema contract |
| `BJX-VALIDATION-001` | `validation` | `bijux-dag-core` | Semantic graph validation failed |
| `BJX-CONFIG-001` | `config` | `bijux-dag-app` | Invalid configuration input |
| `BJX-POLICY-001` | `policy` | `bijux-dag-runtime` | Policy denied requested behavior |
| `BJX-EXEC-001` | `execution` | `bijux-dag-runtime` | Node execution failed |
| `BJX-IO-001` | `io` | `bijux-dag-app` | Filesystem or artifact I/O operation failed |
| `BJX-REPLAY-001` | `replay` | `bijux-dag-app` | Replay mismatch against recorded artifacts |
| `BJX-CACHE-001` | `cache` | `bijux-dag-runtime` | Cache contract or proof mismatch |
| `BJX-COMPAT-001` | `compatibility` | `bijux-dev-dag` | Compatibility contract violation |
| `BJX-INTERNAL-001` | `internal` | `bijux-dag-cli` | Unexpected internal failure path |

The machine-readable authority is
`configs/dag/policy/error_codes.json`. A code missing from that registry is not
a public error identifier even if similar text appears in logs or source.

## Caller Rules

- Branch on the full public code when remediation differs by failure.
- Use the category for aggregation, dashboards, or broad retry policy.
- Preserve the command, exit status, and structured payload in incident
  evidence.
- Treat unknown codes as failures and retain their payload; do not coerce them
  into the nearest known category.
- Do not retry `policy`, `schema`, or `validation` failures without changing
  the request or governed input.

## Change Compatibility

Adding a code is additive only when existing meanings remain unchanged.
Removing a code, changing its category, reassigning its semantic owner, or
giving it a different meaning is an incompatible contract change. Such a
change must update the registry, [Error Contract](../../spec/ERROR_CONTRACT.md),
this page, and both executable error contracts in one reviewable change.

## Implementation And Proof

- `crates/bijux-dag-app/src/commands/output_contract.rs` maps failures into the
  structured error envelope.
- `crates/bijux-dag-app/src/routes/response.rs` preserves numeric process
  status in command responses.
- `crates/bijux-dag-app/tests/error_output_contract.rs` governs structured
  output fields.
- `crates/bijux-dag-app/tests/error_exit_contract.rs` governs parse and
  validation exit behavior.
- `crates/bijux-dev/src/commands/contract_governance.rs` checks registry,
  documentation, and test coverage.

## Related Interfaces

- [Error Model](../architecture/error-model.md)
- [Data Contracts](data-contracts.md)
- [Failure Recovery](../operations/failure-recovery.md)
