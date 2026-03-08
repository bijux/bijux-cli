# App Route To Service Mapping

generated_from: architecture contract source scans

| Route module | Primary service dependency |
| --- | --- |
| `routes/inspect_routes.rs` | run data readers and inspect helpers only |
| `routes/diff_routes.rs` | `replay_service::run_diff_from_dirs` |
| `routes/replay_routes.rs` | `replay_service::run_diff_from_dirs` and replay command response helpers |
| `routes/surface_routes.rs` | capability matrix helpers |
| `routes/plan_routes.rs` | planner lowering and diagnostics helpers |

Boundary intent:

- inspect routes stay free of replay/config service coupling
- replay diff routes centralize diff logic through replay service
- config resolution remains behind config helper functions in `lib.rs`
