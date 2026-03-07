# App Service Boundary Report

Top-level `bijux-dag-app` command dispatch delegates service logic to module boundaries:

- `inspect_service` for run inspection flows
- `replay_service` for semantic run diff and replay equivalence
- `routes::diagnostics_routes` for operator diagnostics surfaces

Boundary rule:

- command parsing and envelope emission stay in `lib.rs`
- business logic and payload assembly stay in service/route modules

