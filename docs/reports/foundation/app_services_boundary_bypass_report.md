# App Services Boundary Bypass Report

generated_from: service-boundary contracts and route-to-service mapping checks.

## Result

No command-family route currently bypasses its intended service boundary in the tracked paths.

## Verified boundaries

- inspect command family uses inspect service pathways.
- replay command family uses replay service pathways.
- diff command family uses replay diff service pathways.
- export/import command family stays on export/import helpers.

## Sources

- `crates/bijux-dag-app/tests/service_boundary_contract.rs`
- `docs/reports/foundation/app_route_to_service_mapping.md`
