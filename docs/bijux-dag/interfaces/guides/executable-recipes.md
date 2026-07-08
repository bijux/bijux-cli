---
title: Executable Recipes Guide
audience: mixed
type: guide
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Executable Recipes Guide

Use the canonical
[Executable Recipes](../executable-recipes.md)
page when you need the full CI-executable command set for DAG operator work.

That page deliberately separates stable operator commands from experimental explicit-path routes and includes the executable example
`bijux-dag explain --json ${RUN_DIR}`.

The release-boundary source of truth for those lane distinctions is the
[Release Boundary](../../foundation/release-boundary.md) page, which is backed
by the machine-readable contract
`contracts/foundation/dag_release_truth_table.v1.json`.
