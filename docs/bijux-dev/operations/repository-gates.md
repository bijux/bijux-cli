---
title: Repository Gates
audience: maintainers
type: operations
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-09
---

# Repository Gates

Use this page when a change is ready for scrutiny and you need to know which
gate families are supposed to prove it safe to merge.

Repository gates matter because `bijux-core` does not treat CI as a ceremonial
green badge. The same evidence should support local review, continuous
integration, and release readiness.

## Gate Layers

- workspace build and test gates
- program-level contract gates for CLI and DAG
- docs structure, link, and build gates
- maintainer suite gates for ownership and policy contracts

## Canonical Commands

```bash
make test
make dag-test
make docs-check
cargo run -q -p bijux-dev --bin bijux-dev-cli -- quickcheck --format json --no-pretty
```

## How To Read A Failure

When a gate fails, classify first before retrying:

1. `layout/docs` failure
2. `contract/schema` failure
3. `runtime/test` failure
4. `automation/workflow` failure

Classification controls which maintainer commands and handbook pages to use
next.

## What A Green Gate Should Mean

| Signal | What it should prove |
| --- | --- |
| local gate success | the change is reproducible on a maintainer workstation |
| CI gate success | the change survives repository automation and baseline environments |
| docs gate success | published guidance still matches the code and file layout |
| contract gate success | public promises still align with behavior and schemas |

## Reader Shortcut

Repeatedly rerunning a failing gate without classification is not diagnosis.
First decide whether the failure is about layout, contracts, runtime, or
automation, then move to the relevant surface.

## Code Anchors

- `makes/rust.mk`
- `makes/dag.mk`
- `makes/docs.mk`
- `crates/bijux-dev/src/suites/`

## Continue Reading

- [Evidence Collection](evidence-collection.md)
- [Quality Policy](../governance/quality-policy.md)
- [Core Testing and Validation](../../bijux-core/governance/testing-and-validation.md)
