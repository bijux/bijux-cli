# ADR: App Routing Shape

## Status

Accepted - March 8, 2026

## Context

`crates/bijux-dag-app/src/lib.rs` had accumulated route handling, response construction,
and mixed command-family branching. This made ownership, review, and release drift control
hard to maintain.

## Decision

The app routing shape is standardized as:

1. `lib.rs` owns CLI parsing, command dispatch, shared helpers, and stable envelope emission.
2. Command-family route handling lives under `crates/bijux-dag-app/src/routes/`.
3. JSON response helper primitives live in `routes/response.rs`.
4. Rendering primitives live in `routes/renderer.rs`.
5. Route complexity and dependency surfaces are tracked by generated reports and enforced by contracts.

## Consequences

1. Route ownership is explicit, with command-family entrypoints mapped to dedicated modules.
2. Reviewers can verify scope drift through architecture reports instead of ad-hoc inspection.
3. File-size and architecture contracts provide release-safe guardrails for future changes.
