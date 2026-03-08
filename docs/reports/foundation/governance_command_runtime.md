# Governance Command Runtime Report

## Scope
Runtime profile targets for governance commands that frequently run in CI.

## Command runtime buckets
- fast: static contracts, taxonomy checks, source-layout checks
- medium: registry/report generation commands
- slow: full evidence synthesis and release bundle assembly

## Immediate optimization candidates
1. Reduce repeated workspace file tree scans in `commands/mod.rs` by sharing collected inventories.
2. Prefer cached JSON registry loads for evidence resolution commands in single-process runs.
3. Keep benchmark and release-evidence heavy commands out of fast lane.
