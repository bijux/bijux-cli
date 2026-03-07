# Runtime execution flow

Execution path:
1. Planner builds deterministic execution plan from graph + runtime config.
2. Executor materializes inputs, computes fingerprints, and resolves adapter dispatch.
3. Adapter execution produces outputs and status.
4. Trace writer persists per-node trace, attempt events, and resolved params.
5. Artifact persistence writes manifest, outputs indexes, provenance, and run summary.

Data-flow contract:
- Planner and policy evaluation are deterministic for identical input state.
- Trace writing is append-only per node attempt and final status.
- Artifact persistence is schema-bound by run manifest and trace schemas.
