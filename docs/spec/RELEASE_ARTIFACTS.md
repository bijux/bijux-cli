# Release Artifacts

## Scope
Defines the expected artifact set for each release.

## Required artifacts
- binaries
- checksums
- schemas snapshot
- docs snapshot
- benchmark summary
- resource profile summary
- compatibility matrix
- known limitations note
- release readiness report
- release evidence bundle

## Related tests
- `crates/bijux-dev-dag/src/commands/mod.rs` (`run_release_readiness_report`, `run_release_evidence_bundle`)

## Versioning and change policy
Artifact set removals are breaking for release consumers and require policy update.
