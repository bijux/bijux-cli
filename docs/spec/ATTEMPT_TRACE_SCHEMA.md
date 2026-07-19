---
title: Attempt Trace Schema
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Attempt Trace Schema

This document defines the stable backend attempt record carried by
`ExecutionAttemptRecord`.

The Rust type is the serialization authority. This path remains stable as the
record evolves; compatibility changes belong in the contract rules and release
notes rather than in parallel version-named documents.

## Record shape

Each attempt record contains:

- `node_id`: stable node identity inside the run
- `attempt`: monotonically increasing attempt number for that node
- `backend_kind`: backend class used for execution
- `status`: terminal `NodeStatus` observed for the attempt
- `exit_code`: process-like exit code when the backend exposes one

## Source of truth

The engine emits these records from `execute_with_backend` after backend
lifecycle completion and cleanup handling.

The `EngineOutcome` schema is:

```json
{
  "attempts": [
    {
      "node_id": "extract",
      "attempt": 1,
      "backend_kind": "Shell",
      "status": "Success",
      "exit_code": 0
    }
  ]
}
```

## Contract rules

- one successful attempt record is emitted per completed node attempt
- failed lifecycle steps do not emit a forged success attempt record
- `backend_kind` must match the backend selected by the capability binding
- `status` and `exit_code` must reflect the observed backend lifecycle result

## Related proof

- `crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs`
- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
- `docs/spec/BACKEND_CONTRACT.md`
