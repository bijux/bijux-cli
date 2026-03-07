# Control-plane foundation

## Scope

`bijux-dev-dag` is the source of truth for repository verification, release verification orchestration, and governance suite execution.

## Required command surfaces

- repo verification
- docs verification
- naming verification
- crate boundary verification
- fixture verification
- artifact contract verification
- release verification
- foundation hardening verification
- ci verification

## Required guard surfaces

- root directory guard
- executable guard
- config verification
- schema verification
- docs taxonomy verification
- test trust verification
- runtime taxonomy verification

## Foundation suite

Suite id `foundation-verification` validates that core governance suites are registered in the repo suite index and the SSOT contract files exist.

## Foundation hardening suite

Command `foundation-hardening` executes curated high-trust suites listed in `configs/suites/foundation_hardening.json`.

## SSOT rule

When a governance policy is changed, `bijux-dev-dag` command implementation and the owning contract documentation must be updated in the same change.
