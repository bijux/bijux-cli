# Kernel Contraction Objectives Report

This report tracks completion of the first twenty kernel-contraction objectives.

1. Canonical mission page linked from root README: `README.md` -> `docs/spec/MISSION_STATEMENT.md`.
2. Root README implemented/modeled/simulated matrix: `README.md`.
3. Generated kernel-only runtime module report: `docs/reports/foundation/KERNEL_OWNED_MODULES_REPORT.md`.
4. Generated non-kernel runtime module report: `docs/reports/foundation/RUNTIME_NON_KERNEL_MODULES_REPORT.md`.
5. Core runtime references separated from control-plane/model references in docs index: `docs/INDEX.md`.
6. Guard: kernel modules must not depend on app crates: `dependency_boundary_contracts.rs`.
7. Guard: kernel modules must not depend on dev-governance crates: `dependency_boundary_contracts.rs`.
8. Guard: core must not depend on runtime: `no_runtime_in_core.rs`.
9. Guard: runtime must not depend on CLI parsing crate families: `runtime_contraction_contracts.rs` and `no_cli_in_runtime.rs`.
10. Guard: app must not import runtime internals: `dependency_boundary_contracts.rs`.
11. Core public API review surface produced: `docs/reports/foundation/core_PUBLIC_API_SHRINK_REPORT.md`.
12. Runtime public API review surface produced: `docs/reports/foundation/runtime_PUBLIC_API_SHRINK_REPORT.md`.
13. Core public API shrink report generated from source: `docs/reports/foundation/core_PUBLIC_API_SHRINK_REPORT.md`.
14. Runtime public API shrink report generated from source: `docs/reports/foundation/runtime_PUBLIC_API_SHRINK_REPORT.md`.
15. Kernel allowed dependency contract documented and tested: `docs/spec/KERNEL_ALLOWED_DEPENDENCIES.md`, `dependency_boundary_contracts.rs`.
16. Runtime allowed dependency contract documented and tested: `docs/spec/RUNTIME_ALLOWED_DEPENDENCIES.md`, `dependency_boundary_contracts.rs`.
17. Dev-governance allowed dependency contract documented and tested: `docs/spec/DEV_GOVERNANCE_ALLOWED_DEPENDENCIES.md`, `dependency_boundary_contracts.rs`.
18. Kernel-adjacent public naming guard for modeled/future language: `dependency_boundary_contracts.rs`.
19. Runtime contract-backed vs documented-only report generated: `docs/reports/foundation/RUNTIME_CONTRACT_BACKING_REPORT.md`.
20. Runtime operator-facing vs internal-only report generated: `docs/reports/foundation/RUNTIME_OPERATOR_SURFACE_REPORT.md`.
