---
title: Export Bundle Evolution Rulebook
audience: mixed
type: spec
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-06
---

# Export Bundle Evolution Rulebook

Export and proof bundle evolution must stay explicit about supported,
unsupported past, and unsupported future formats.

## Scope

This rulebook governs export and proof bundle version surfaces backed by:

- `evidence/compat/export_bundle/`
- `evidence/compat/proof_bundle/`

## Evolution rules

- current export bundle versions must import successfully
- unsupported past or future bundle versions must be rejected explicitly
- proof bundle version lanes must stay aligned with replay-bundle
  compatibility policy
- diagnostics bundle versions must evolve independently from replay-bundle
  compatibility because diagnostics capture is not an importable replay surface

## Related tests

- `crates/bijux-dag-app/tests/version_fixture_contracts.rs`
- `crates/bijux-dev/tests/foundation_version_compatibility_lanes_contracts.rs`

## Versioning and change policy

Any incompatible bundle-format change must update this rulebook, the lane
contract, and the supported or refused fixtures in the same change.
