# Graph Diff Semantics

Graph diff classifies changes into:

- semantic: changes that alter graph identity or execution behavior
- cosmetic: formatting/order/comment changes that do not alter graph identity

Canonical graph bytes are the source of truth for semantic comparison.
