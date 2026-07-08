---
title: Change Validation
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-09
---

# Change Validation

Use this page when a CLI change is real and the next question is not "did the
code compile," but "what proof is required before anyone should trust it?"

Change validation is the minimum evidence package for a `bijux-cli` behavior
change. It exists to stop silent contract drift in command grammar, payload
shape, stream usage, help output, and exit behavior.

## Validation Checklist

1. classify whether command grammar, output, or exit semantics changed
2. execute targeted routing and integration suites
3. run architecture boundary tests when module ownership shifts
4. verify docs structure and handbook consistency
5. include explicit compatibility notes when callers may be affected

## Validation Commands

```bash
cargo test -p bijux-cli routing::
cargo test -p bijux-cli integration::
cargo test -p bijux-cli architecture::
make docs-check
```

## What Validation Should Produce

| Surface | What reviewers should walk away with |
| --- | --- |
| targeted tests | confidence that the owning lane actually exercised the changed contract |
| docs gate | confidence that handbook claims and file structure still match the tree |
| compatibility notes | explicit warning when scripts, plugins, or operators may need to adapt |

## Code Anchors

- `crates/bijux-cli/tests/routing/`
- `crates/bijux-cli/tests/integration/`
- `crates/bijux-cli/tests/architecture/`
- `makes/docs.mk`

## Reader Shortcut

If a reviewer cannot point to the exact test and doc surfaces that changed with
the behavior, validation is still incomplete. Broad green status is not enough
when the affected contract is narrow and specific.

## Continue Reading

- [Definition of Done](definition-of-done.md)
- [Compatibility Commitments](../interfaces/compatibility-commitments.md)
- [Test Strategy](test-strategy.md)
