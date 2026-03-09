# Internal Differences: Legacy Python vs Rust-Backed Runtime

User-facing command behavior is kept compatible.

Internal differences:

- Execution engine is Rust-backed.
- Python package serves as facade + compatibility layer.
- Extension loading failures fall back to subprocess delegation.
- Legacy alias APIs emit deprecation warnings.
