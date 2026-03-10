# Rustdoc Primary Code Docs Policy

Rustdoc is the primary code documentation path for `bijux-cli`.

Rules:
- Public Rust APIs must be documented in Rust source using Rustdoc comments.
- Website/API docs must link to Rustdoc-owned truth instead of duplicating API semantics.
- `bijux dev cli rustdoc audit` is the maintainer-facing authority for Rustdoc health.
- Release truth may not claim code-doc completeness without Rustdoc audit artifacts.
