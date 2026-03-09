# Documentation Governance and Truth Boundary

Status: accepted
Owner: platform documentation guild
Date: 2026-03-09

## Decision
Documentation is governed by strict source-of-truth boundaries. Normative contracts live in `docs/spec/`; explanatory and operational material must reference those contracts and must not duplicate them.

## Consolidated from
- 20260308-documentation-truth-policy.md
- 20260309-documentation-governance-alignment.md

## Consequences
- Documentation drift checks are expected in control-plane governance.
- Root docs are entrypoints only.
- Contract text duplication is treated as governance debt.
