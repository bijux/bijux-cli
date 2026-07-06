---
title: Internal Hook Promotion Checklist
audience: maintainer
type: spec
status: canonical
owner: bijux-dag-maintainers
last_reviewed: 2026-07-06
---

# Internal Hook Promotion Checklist

Use this checklist before promoting an internal runtime hook into a supported
extension point.

## Scope

This checklist governs promotion decisions for hooks modeled by
`InternalHookPromotionChecklist` in
`crates/bijux-dag-runtime/src/internal/ext/extension_catalog.rs`.

## Required promotion gates

An internal hook is ready for promotion only when all of the following are
true:

- `has_contract_doc`
- `has_versioning_policy`
- `has_negative_tests`
- `has_failure_isolation`

If any gate is false, the hook remains internal.

## Maintainer review prompts

- does the hook have a durable contract document that names ownership, scope,
  and compatibility expectations
- does the contract define how incompatible changes are versioned
- do negative tests prove invalid inputs, broken integrations, and missing
  capabilities are rejected
- do failure tests prove hook malfunction cannot be mistaken for a healthy
  engine outcome

## Primary proof

- `crates/bijux-dag-runtime/src/internal/ext/extension_catalog.rs`
- `crates/bijux-dag-runtime/tests/extension_catalog_contracts.rs`
