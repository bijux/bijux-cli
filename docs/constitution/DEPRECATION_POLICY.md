# Deprecation Policy

## Purpose
Define how command and behavior changes are introduced without breaking users unexpectedly.

## Scope
This policy governs deprecation announcement, message format, and removal timing.

## Core Concepts
- Deprecation is a compatibility process, not a one-time warning.

## Invariants
- A deprecated behavior remains functional during its announced deprecation window unless security risks require immediate action.
- Deprecation notices must include replacement guidance and removal target version.
- Removal occurs only in a release that satisfies the announced policy window.

## Deprecation Message Format
Use this stable message template in text mode and structured diagnostics:

`DEPRECATED: <subject> is deprecated and will be removed in <version>. Use <replacement>. Reference: <url>.`

## Minimum Timeline
- Announce in release notes and command output diagnostics in one release.
- Keep behavior available for at least one subsequent minor release in the same major line.
- Remove only at the announced boundary.

## Failure Modes
- Removing behavior without deprecation notice is a policy violation.

## Design Rationale
- Predictable deprecation windows reduce CI breakage and migration cost.

## Non-Goals
- Preventing all command evolution.
