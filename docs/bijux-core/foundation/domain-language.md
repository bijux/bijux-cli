---
title: Domain Language
audience: mixed
type: foundation
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-07
---

# Domain Language

This repository repeats a small set of terms across the CLI, DAG, repository,
and maintainer handbooks. Use this page when the words sound familiar but the
ownership behind them is still blurry.

## Core Terms

- `repository handbook`: root docs for cross-program rules and ownership
- `product handbook`: CLI or DAG docs for owned runtime behavior
- `maintainer handbook`: docs for repository-health automation and release work
- `package`: the concrete code ownership boundary under `crates/`
- `contract`: machine-checkable rule or schema that other surfaces rely on
- `evidence`: outputs, reports, or checks that support a review or release

## Why The Vocabulary Matters

- It helps readers move between handbooks without re-learning the structure.
- It keeps product behavior, repository rules, and maintainer automation from
  being described as the same thing.
- It gives reviews a stable way to say who owns what.

## Naming Rule

Prefer names that still explain intent when read out of context two years
later. Avoid labels that only make sense relative to a temporary migration or
iteration.

## What This Page Is Not Saying

- It is not prescribing prose style for every document.
- It is not freezing every word in the repository forever.
- It is not replacing package pages or handbooks when you need concrete detail.

## Continue Reading

- [Package Map](package-map.md)
- [Change Principles](change-principles.md)
- [Repository Scope](repository-scope.md)
