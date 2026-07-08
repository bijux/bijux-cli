---
title: Performance and Scaling
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Performance and Scaling

Use this page when the CLI feels correct but too slow, too heavy, or too
fragile under larger local state and plugin inventories.

Performance in `bijux-cli` is about predictable latency and bounded behavior,
not just raw speed. Operators should be able to reason about which surfaces
scale cleanly and which ones deserve caution as local complexity grows.

## Performance Hotspots

- parser normalization over large argument vectors
- plugin registry discovery and health checks
- history and memory file scanning for large local state files
- rendering very large structured payloads
- delegated command invocation overhead

## Evidence Route

When performance wording changes on this page, refresh maintainer evidence first
with `bijux-dev-dag performance-evidence-report` and confirm the current
scenario metadata in `evidence/perf/metadata.json`.

## What To Watch First

| Surface | Why it becomes expensive |
| --- | --- |
| parser and route normalization | large argv shapes and suggestion work can add surprising overhead |
| plugin discovery | install health and manifest loading grow with plugin count |
| history and memory inspection | local state size directly affects scan and render cost |
| structured output rendering | large payloads amplify formatting and serialization work |
| delegated invocations | external command startup can dominate perceived latency |

## Code Anchors

- `crates/bijux-cli/src/routing/parser.rs`
- `crates/bijux-cli/src/features/plugins/discovery.rs`
- `crates/bijux-cli/src/features/history/operations.rs`
- `crates/bijux-cli/src/shared/output.rs`
- `crates/bijux-cli/src/interface/repl/`

## Scaling Rules

- keep telemetry fields bounded and truncation-aware
- separate slow integration contracts from default fast test gates
- avoid unbounded data joins in diagnostics payloads
- prefer streaming or targeted queries for large state files

## Reader Shortcut

If a command becomes slow only when local state, plugins, or payload size
grows, treat that as a scaling surface, not random noise. The slowdown usually
belongs to a specific owned hotspot.

## Continue Reading

- [Test Strategy](../quality/test-strategy.md)
- [Known Limitations](../quality/known-limitations.md)
- [Architecture Risks](../architecture/architecture-risks.md)
