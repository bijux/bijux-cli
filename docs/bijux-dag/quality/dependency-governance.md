---
title: Dependency Governance
audience: maintainers
type: quality
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-19
---

# Dependency Governance

A dependency change can alter graph parsing, canonical identity, retained
schemas, scheduler behavior, subprocess boundaries, or release order without
changing much local code. Treat the dependency's owned behavior as part of the
change, not as an implementation detail hidden by `Cargo.lock`.

## Authorities

| Concern | Authority | What it governs |
| --- | --- | --- |
| shared versions and features | root `Cargo.toml` | workspace dependency intent |
| package-specific use | each crate's `Cargo.toml` | which owner consumes the dependency and with which extra features |
| resolved graph | `Cargo.lock` and `cargo metadata --locked` | exact transitive versions used by repository gates |
| DAG package direction | `contracts/foundation/dag_dependency_direction.v1.json` and boundary tests | allowed workspace edges |
| narrow forbidden edges | `configs/dag/policy/dependency_rules.json` | explicit edges rejected by the maintainer dependency guard |
| sources, licenses, yanks, duplicates | `configs/rust/deny.toml` | cargo-deny policy |
| Rust advisories | `cargo audit` plus `audit-allowlist.toml` | known-vulnerability decisions |
| managed upgrades | `.github/dependabot.yml` | weekly Cargo update proposals |

No single row replaces the others. A lockfile proves resolution, not
architectural permission. A legal source and license do not prove semantic
compatibility. A passing package test does not prove the public release graph
still publishes in dependency order.

## Risk Classification

| Dependency role | Typical examples | Required review |
| --- | --- | --- |
| graph and configuration parsing | `serde`, `serde_json`, `serde_yaml`, `toml` | accepted/rejected fixtures, unknown-field behavior, defaults, canonical output |
| identity and integrity | `sha2`, `hex`, archive and compression crates | fingerprints, cache keys, proof records, bundle round trips |
| time and scheduling | `chrono`, `chrono-tz`, `croner` | timezone, boundary-date, deterministic schedule, and serialization cases |
| process and signal handling | `ctrlc`, runtime adapter dependencies | cancellation, exit classification, cleanup, retained failure evidence |
| network and TLS | `reqwest` and enabled TLS features | timeout, certificate, redirect, response bounds, and offline refusal |
| CLI parsing | `clap`, `clap_complete` | generated reference, command inventory, completion, and exit behavior |
| test or maintainer support | `tempfile`, governance dependencies | production-edge exclusion and artifact containment |

Feature changes deserve the same review as version changes. Enabling a default
feature can add a network stack, native library, runtime, or source that was
previously absent.

## Review Procedure

1. Identify the direct owner and the behavior obtained from the dependency.
2. Inspect the lockfile diff, including new transitive packages, features,
   sources, licenses, and duplicate versions.
3. Run `cargo metadata --locked --format-version 1` and confirm workspace
   dependency direction did not change.
4. Add or update focused tests at the behavior boundary affected by the
   upgrade.
5. Run the package-boundary and audit gates.
6. Record any user-visible schema, identity, diagnostic, or compatibility
   change in the owning documentation and changelog.

Do not combine unrelated dependency upgrades merely to reduce pull-request
count. A narrow update makes regressions attributable and lets a reviewer
evaluate one compatibility surface at a time.

## Required Gates

```bash
cargo test -p bijux-dev --test foundation_dag_dependency_direction_contracts
cargo test -p bijux-dev --test dependency_boundary_contracts
cargo test -p bijux-dev --test foundation_workspace_package_boundary_contracts
make audit
```

`make audit` first validates repository allowlist and deviation governance,
then runs cargo-deny for bans, licenses, and sources and cargo-audit for
advisories. It requires the repository-pinned helper versions installed by the
documented toolchain path.

These gates are necessary but not sufficient. Run the focused product tests
for the dependency role from the classification table.

## Exceptions

`audit-allowlist.toml` is the only repository advisory exception record. Every
entry requires a RustSec id, rationale, owner, expiry, and review link.
Expired or malformed records fail the maintainer security command.

`configs/rust/deny.deviations.toml` governs standards-level cargo-deny
deviations and requires an owner, reason, expiry, and `bijux-std` review link.
Do not suppress an advisory or broaden a source/license policy in command-line
flags or a workflow.

## Release Impact

For a public DAG crate, confirm `cargo package --list` and publish dry-run
behavior through the release-validation suite. An upgrade that compiles from a
workspace path can still fail from a packaged crate because features, files,
or dependency versions differ at the registry boundary.

If a change affects serialized evidence or identity, existing retained runs
and replay bundles are compatibility fixtures. Preserve refusal behavior when
old evidence cannot be accepted safely; do not silently reinterpret it.

## Related Guidance

- [Dependency Direction](../architecture/dependency-direction.md)
- [Change Validation](change-validation.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
- [Release Validation Suite](../../bijux-dev/operations/release-validation-suite.md)
