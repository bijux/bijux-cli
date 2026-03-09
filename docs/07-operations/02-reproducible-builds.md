# Reproducible Builds

## Purpose
Define how to produce deterministic build and test results across environments.

## Context
Reproducibility is required for trustworthy replay/diff outcomes and reliable release decisions.

## Explanation
Reproducible build controls:
- pin Rust toolchain version and lockfile.
- pin CI container/image base where feasible.
- avoid non-deterministic build inputs (floating dependency versions, mutable environment defaults).

Runtime reproducibility controls:
- explicitly set locale and timezone assumptions in CI.
- avoid hidden environment coupling in DAG node commands.
- keep fixture inputs versioned and immutable per test lane.

Cache strategy:
- use dependency caching for speed, but key cache by lockfile/toolchain.
- never treat cache hits as correctness proof.
- fallback to clean build path on cache inconsistency.

Artifact caching guidance:
- separate dependency cache from build-output cache.
- cache keys should include lockfile hash, toolchain version, and target triple.
- invalidate caches when compiler/toolchain policy changes.
- periodically run cache-bypass builds to detect hidden cache coupling.

Drift detection:
- periodically run clean builds without cache.
- compare key output hashes for release-critical artifacts.
- classify drift as code/input/environment before mitigation.

## Examples
```bash
rustup override set 1.80.1
cargo build --workspace --locked
cargo test --workspace --locked
```

```yaml
# CI cache key example (conceptual)
key: cargo-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-rust-1.80.1
```

```text
Reproducibility checklist:
- toolchain pinned
- lockfile committed
- clean lane passes
- artifact hash comparison stable
```

## Guarantees
- Reproducibility controls are explicit and operationally enforceable.
- Build drift can be detected and classified through deterministic checks.
- Release-critical workflows can require clean, pinned execution paths.
- Cache strategy explicitly favors correctness over speed when they conflict.

## Limitations
- Absolute binary identity can still vary across platform/toolchain families.
- External services used by commands can introduce non-reproducible behavior.
- This document does not define cryptographic provenance attestation format.

## Related
- `docs/07-operations/01-ci-integration.md`
- `docs/07-operations/05-backend-support.md`
- `docs/08-development/02-testing-strategy.md`
- `docs/06-specification/07-replay-semantics.md`
