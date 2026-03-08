# Runtime `pub use` Audit

generated_from: `crates/bijux-dag-runtime/src/lib.rs`

## Summary

- audited all runtime `pub use` blocks for low-value re-exports.
- no removals in this pass; entries are either:
  - required for stable runtime contract surface, or
  - already tracked as quarantined broad surfaces in runtime ownership policy.

## Follow-up Policy

- any new `pub use` must link to a contract or owning report before merge.
