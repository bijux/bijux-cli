# Run History Integrity Guarantees

Status: accepted
Owner: runtime maintainers
Date: 2026-03-09

## Decision
Run history is treated as authoritative operational evidence with strict integrity and corruption-recovery behavior.

## Consolidated from
- 20260308-run-history-guarantees.md

## Consequences
- Run identity and ancestry remain stable surfaces.
- Corruption handling must not panic and must stay diagnosable.
