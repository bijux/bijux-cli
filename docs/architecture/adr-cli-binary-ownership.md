# ADR: CLI Binary Ownership

## Status
Accepted

## Decision
`bijux-cli` is the sole owner of the `bijux` executable contract across all distribution channels.

## Context
Multiple package channels (`cargo install bijux-cli`, `cargo install bijux`, `pip install bijux-cli`, `pip install bijux`) are supported for compatibility. Without a single ownership rule, runtime ambiguity and behavioral drift become likely.

## Consequences

- All channels must expose the same executable name: `bijux`.
- Compatibility alias packages may exist but cannot define divergent command behavior.
- `bijux cli paths` and `bijux cli doctor` remain the source of truth for active binary diagnostics.
- Release and packaging workflows must preserve checksum and artifact manifest parity across channels.

