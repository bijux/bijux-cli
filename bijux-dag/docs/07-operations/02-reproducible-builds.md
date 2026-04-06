# Reproducible Builds

Reproducible builds in bijux-dag mean build and validation outcomes remain classification-equivalent when the declared input set is unchanged.

## Required reproducibility inputs

Track and pin these inputs:
- compiler/toolchain version,
- dependency lockfiles,
- build image or host baseline,
- target triple,
- environment variables that affect build/runtime behavior,
- fixture/input versions used in tests and replay checks.

Changing any required input invalidates prior reproducibility claims until re-verified.

## Build discipline

Minimum operator discipline:
- run `--locked` dependency resolution,
- pin toolchain per repository,
- keep build and test commands deterministic,
- separate dependency cache from build-output cache.

Cache keys should include lockfile hash, toolchain version, and target triple.

## What reproducible-build claims mean

A reproducible-build claim means:
- under the declared input set and environment envelope,
- repeated builds produce equivalent classification for required evidence scopes.

It does not mean:
- byte-identical binaries across all platforms,
- equivalence on unsupported backends,
- immunity to external service variability.

## Common drift causes

Most drift incidents come from:
- floating dependencies or uncommitted lockfile changes,
- mutable base images,
- hidden timezone/locale differences,
- test commands that call external network resources,
- stale caches masking source or toolchain changes.

## Operator checklist

Before promotion:
- confirm pinned toolchain and committed lockfile,
- execute one clean build lane with cache bypass,
- compare release-critical artifact identities,
- classify any drift as code/input/environment,
- block promotion when drift classification is unresolved.

## Guarantees

- Reproducibility inputs are explicit and auditable.
- Drift sources can be classified instead of guessed.
- Clean-lane checks prevent cache-only confidence.

## Non-guarantees

- No universal byte-identity guarantee across heterogeneous hosts.
- No guarantee for unpinned external services.

## Next reading

- [CI integration](docs/07-operations/01-ci-integration.md)
- [Replay semantics contract](docs/06-specification/07-replay-semantics.md)
- [Backend support](docs/07-operations/05-backend-support.md)
