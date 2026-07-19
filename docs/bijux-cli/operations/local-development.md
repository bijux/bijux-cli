---
title: Local Development
audience: contributors
type: operations
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# Local Development

Use this page while editing `bijux-cli`. The goal is a short loop that runs the
actual binary, reproduces one behavior, and exercises the smallest owning test
lane before broader validation.

## Prepare Once

From the repository root:

```bash
make bootstrap
make doctor
```

Cargo products remain under `artifacts/rust/target/`; Python tooling and caches
remain under `artifacts/python/`.

## Edit Loop

1. Identify the command, payload, or ownership boundary being changed.
2. Run the current binary with the smallest input that exposes the behavior.
3. Run the focused test that should fail if the behavior regresses.
4. Inspect snapshots or structured output semantically.
5. Update operator documentation when user-visible meaning changes.

For command behavior:

```bash
cargo run -q -p bijux-cli --bin bijux -- status --format json --no-pretty
cargo nextest run -p bijux-cli -E 'test(/routing/)'
cargo nextest run -p bijux-cli -E 'test(/integration/)'
```

Use a narrower package, binary, test target, or test-name expression when the
owner is already known. A focused command is diagnostic evidence, not a
workspace pass.

## Choose The Owning Test

| Change | First proof |
| --- | --- |
| route parsing, aliases, or suggestions | routing test nearest the parser rule |
| command behavior or output envelope | integration test invoking the real route |
| plugin lifecycle or mounted app behavior | lifecycle integration contract |
| module dependency direction | architecture boundary test |
| Python launcher or native bridge | focused Python package test plus native parity check |
| handbook command example | focused behavior test plus `make docs-check` |

When a test does not exist at the owning boundary, add one before relying on a
broad suite that only exercises the behavior incidentally.

## Inspect Output Honestly

For structured output, verify:

- stdout contains the complete machine-readable envelope;
- stderr contains diagnostics or progress only;
- `ok`, error code, and process exit status agree;
- ordering is deterministic where snapshots or callers rely on it;
- secrets and local absolute paths are absent unless explicitly part of the
  contract.

For human output, verify meaning and remediation rather than preserving spacing
that has no compatibility value.

## Leave The Loop

The local loop is complete when the focused failure is fixed and the owning
test passes. It is not merge evidence by itself. Move to
[Change Validation](../quality/change-validation.md) to select compatibility,
documentation, and repository gates for the completed change.

## Code Anchors

- routing: `crates/bijux-cli/tests/routing/`
- behavior: `crates/bijux-cli/tests/integration/`
- ownership: `crates/bijux-cli/tests/architecture/`
- Python bridge: `crates/bijux-cli-python/tests/python/`
