---
title: Release Notes Template
audience: maintainers
type: operations
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-05
---

# Release Notes Template

Use this template when preparing repository release notes for a tagged
publication. Fill every section with evidence-backed statements only.

## Required Inputs

- released version and tag
- commit range or merged change set
- compatibility evidence for every public behavior change
- docs and migration links for every operator-visible change
- known limitations that remain unresolved at release time

## Template

```md
# Release <version>

## Summary

- explain what changed in one or two evidence-backed bullets

## Public Behavior Changes

- list operator-visible command, API, schema, or artifact changes
- link the contract, handbook, or migration page that explains each change

## Compatibility Notes

- state whether the release is fully compatible, conditionally compatible, or
  intentionally breaking
- name the affected surface and the required operator response

## Migration Steps

- list exact commands, config changes, or rollout steps operators must perform
- say explicitly if no migration is required

## Known Limitations

- link active limitations that still apply to this release
- explain whether the release changes the workaround or release target

## Evidence

- list the verification artifacts, benchmark evidence, or release reports that
  support the notes above
```

## Writing Rules

- do not claim behavior that the release evidence does not prove
- do not omit operator action when a change requires one
- do not summarize hidden or simulated surfaces as shipped capabilities
- update limitation links whenever the release changes their impact or
  workaround

## Next Reads

- [Release and Versioning](release-and-versioning.md)
- [Known Limitations](../../bijux-dag/quality/known-limitations.md)
