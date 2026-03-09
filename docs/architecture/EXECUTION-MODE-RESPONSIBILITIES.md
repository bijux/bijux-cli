# Execution Mode Responsibilities

## Local executor
- Launches local subprocess or in-process execution.
- Uses local filesystem-backed artifact storage.
- Provides baseline correctness behavior.

## Container executor
- Interprets container execution contract fields.
- Maps local roots to container mounts.
- Enforces container output declaration and timeout classification.

## Remote/Kubernetes executor
- Submits work to remote control plane and tracks external identity.
- Maps remote statuses into local node/attempt state model.
- Handles artifact and observability handoff contracts.

## Shared responsibilities across all modes
- planner and scheduler semantics
- retry accounting and attempt state transitions
- policy enforcement for declared effects
- run-dir and trace contract integrity

## Current implementation boundary
- Local/process-like execution path is implemented.
- Container and remote execution are contract/simulation surfaces and must not
  be described as production-ready in normative docs.
