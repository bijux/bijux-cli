---
title: Local Development
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Local Development

Use this page when you are changing `bijux-cli` locally and need a development
loop that keeps command behavior, tests, and handbook updates moving together.

Local development should feel fast, but not vague. The point is to shorten the
distance between a code edit and the exact runtime, test, and documentation
evidence that proves the edit is safe.

## Local Development Loop

- run command behavior locally through `cargo run -p bijux-cli --bin bijux -- ...`
- run focused tests in affected routing, integration, or architecture suites
- update handbook pages when user-facing behavior changes
- rerun docs and contract checks before commit

## Typical Commands

```bash
cargo run -p bijux-cli --bin bijux -- status
cargo test -p bijux-cli routing::
cargo test -p bijux-cli integration::
make docs-check
```

## What A Good Local Loop Produces

| Step | Why it matters |
| --- | --- |
| local command run | confirms the edited behavior exists outside the editor |
| targeted tests | proves the owning contract lane still holds |
| docs update | keeps readers from learning stale behavior |
| docs and contract checks | catches publication and structure drift before commit |

## Code Anchors

- `crates/bijux-cli/tests/routing/`
- `crates/bijux-cli/tests/integration/`
- `crates/bijux-cli/tests/architecture/`
- `makes/docs.mk`

## Development Rules

- treat golden/snapshot changes as reviewable contract changes
- avoid mixing unrelated behavior and documentation edits in one commit
- keep commits scoped to one understandable runtime concern

## Reader Shortcut

If the only proof for a local change is "it seemed fine when I tried it once,"
the loop is too weak. A serious CLI edit should leave behind command evidence,
targeted tests, and updated handbook language.

## Continue Reading

- [Common Workflows](common-workflows.md)
- [Test Strategy](../quality/test-strategy.md)
- [Change Validation](../quality/change-validation.md)
