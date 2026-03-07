# Cleanup backlog

- fix runtime compile errors in `crates/bijux-dag-runtime/src/lib.rs` (duplicate imports/re-exports, trait bounds, moved-value usage)
- reconcile conflicting type exports and duplicate symbol definitions across runtime modules
- restore successful `release verify` execution path after runtime compile stabilization
- rerun battle workflow harness and strict verification commands after compile fixes
- generate trend-based historical evidence for foundation metrics
