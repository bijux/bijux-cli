# ADR renovation alignment

## Context

Repository cleanup introduced stronger boundaries for runtime scope, evidence governance, and test trust enforcement.

## Decision

- use a single docs/config governance policy to block inflation and unsupported claims
- require spec-to-code-and-test ownership mapping for normative contracts
- keep modeled/future behavior isolated from current implemented capability claims

## Consequences

- scorecard-style self-evaluation documents are removed from normative index surfaces
- docs-root and config inventory checks become release-gating governance controls
- future architectural changes must update governance mappings and reports in the same change set
