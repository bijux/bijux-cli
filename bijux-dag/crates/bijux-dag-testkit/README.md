# bijux-dag-testkit

Shared test fixtures, builders, and assertion utilities for DAG crates and top-level suites.
Responsibility: Shared deterministic test fixtures, builders, and assertion helpers for workspace crates.

## Why this crate exists
This crate centralizes reusable test fixtures and helpers so contract tests remain deterministic and consistent.

## What must never enter this crate
- Production command routing.
- Runtime state-machine ownership.
- Release governance decision policy.

See [CONTRACT.md](./CONTRACT.md).
