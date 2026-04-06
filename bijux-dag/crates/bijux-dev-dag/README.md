# bijux-dev-dag

Repository control-plane for governance, contract checks, and release verification workflows.
Responsibility: Repository governance control-plane, suite orchestration, and release verification automation.

## Why this crate exists
This crate enforces repository governance contracts and provides release and quality verification tooling.

## What must never enter this crate
- Authoritative runtime execution semantics.
- Core DAG semantic ownership.
- Production artifact storage behavior.

See [CONTRACT.md](./CONTRACT.md).
