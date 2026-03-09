# Module Ownership Map

## Ownership domains
- kernel: `crates/bijux-dag-core`
- runtime: `crates/bijux-dag-runtime`
- adapters: `crates/bijux-dag-runtime/src/backend`
- evidence: `evidence/`, `docs/reports/foundation/`
- app: `crates/bijux-dag-app`, `crates/bijux-dag-cli`
- governance: `crates/bijux-dev-dag`

## Boundary rules
- kernel modules must not depend on app, CLI, or governance modules.
- app and governance modules must consume kernel/runtime contracts, not redefine them.
- evidence reports must cite source contracts and tests.
