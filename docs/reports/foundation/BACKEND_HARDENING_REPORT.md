# Backend Hardening Report

## Purpose

This report records the repository surfaces that currently harden backend
binding, lifecycle sequencing, attempt evidence, and engine/backend ownership.

## Guarded surfaces

- contract: `docs/spec/BACKEND_CONTRACT.md`
- engine contract: `docs/spec/EXECUTION_ENGINE_CONTRACT.md`
- attempt schema: `docs/spec/ATTEMPT_TRACE_SCHEMA_V0.1.md`
- architecture: `docs/bijux-dag/architecture/engine-backend-responsibilities.md`
- runtime implementation: `crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs`
- conformance tests: `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
- engine proof: `crates/bijux-dag-runtime/tests/engine_flow_contract.rs`
- maintainer guard: `crates/bijux-dev/tests/backend_hardening_contracts.rs`

## Current hardening stance

- backend capability mismatch must fail binding before lifecycle work starts
- declared output targets must be authorized before backend launch so malformed
  paths never become writable targets
- cleanup must remain explicit even when prepare, launch, observe, or finalize
  fails
- undeclared outputs must fail finalization instead of entering durable evidence
- new backend implementations are blocked until conformance stays explicit and
  passing
