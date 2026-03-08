# Naming philosophy

Names are long-lived contracts.

A good name must survive structural refactors, personnel changes, and release cycles. Naming is treated as a correctness concern because it shapes module boundaries, ownership, and operator expectations.

## Principles

- encode semantics, not ambition
- optimize for future readers over current authors
- avoid transient project-management vocabulary in product surfaces
- avoid overloaded terms across runtime, artifacts, and scheduler domains

## Decision test

A name is acceptable when:

- it describes behavior without release-context knowledge
- it remains accurate under foreseeable implementation changes
- it can be mapped to one glossary entry or one contract section
