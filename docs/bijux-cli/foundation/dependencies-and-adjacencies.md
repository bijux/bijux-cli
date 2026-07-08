---
title: Dependencies and Adjacencies
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-04-06
---

# Dependencies and Adjacencies

Use this page when you want to understand which dependencies can actually
change `bijux` behavior and which nearby product areas the CLI should not
absorb.

Most dependency updates are routine. The ones that matter are the ones that can
change command grammar, payload shape, help output, compatibility checks, or
plugin lifecycle behavior.

## What Changes Reader-Facing Behavior

| Surface | Why it matters |
| --- | --- |
| command parsing | parsing libraries decide how users express intent and how help output is shaped |
| payload contracts | serialization libraries define what scripts and operators can safely consume |
| plugin compatibility | version comparison and manifest validation decide which extensions can mount |
| error surfaces | runtime error handling changes what operators see when commands fail |
| adjacent product boundaries | dependency drift can blur the line between `bijux`, DAG tools, and repository governance |

## Dependencies With Real Contract Pressure

- `clap`: defines command grammar and help/usage behavior
- `serde` and `serde_json`: define payload and contract serialization behavior
- `schemars`: emits schema assets for envelope and manifest contracts
- `semver`: validates plugin compatibility ranges against host versions
- `thiserror` and `anyhow`: shape internal/runtime-facing errors

## What Still Belongs Somewhere Else

- DAG runtime behavior and proofs stay in `bijux-dag`
- repository-level governance and docs topology stay in `bijux-core`
- maintainer automation and gate orchestration stay in `bijux-dev`

## What This Page Is Not Saying

- It is not arguing that every dependency change is high risk.
- It is not saying the CLI should own workflow execution semantics.
- It is not replacing release notes or compatibility documentation for exact
  version commitments.

## Code Anchors

- `crates/bijux-cli/Cargo.toml`
- `crates/bijux-cli/src/contracts/schema.rs`
- `crates/bijux-cli/src/contracts/plugin.rs`
- `crates/bijux-cli/src/interface/cli/dispatch/help.rs`

## Dependency Review Rule

Any dependency update that changes parser grammar, payload encoding, schema
shape, or semver comparison behavior requires targeted tests and handbook updates
in the same pull request.

## Continue Reading

- [Dependency Direction](../architecture/dependency-direction.md)
- [Dependency Governance](../quality/dependency-governance.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
