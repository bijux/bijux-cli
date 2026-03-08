# Environment capability guide

This guide defines what DAG authors can expect from each deployment profile.

## local

- backend: local
- artifact store: filesystem
- network isolation: optional
- recommended for development and deterministic debugging

## ci

- backend: subprocess
- artifact store: filesystem
- network isolation: enforced
- recommended for repeatable validation and release verification

## cluster

- backend: kubernetes
- artifact store: object storage
- network isolation: enforced
- recommended for production-scale orchestration

## Authoring guidance

- Declare explicit resource requirements when targeting cluster execution.
- Avoid backend-specific assumptions in DAG core spec fields.
- Use policy overlays for backend-specific restrictions instead of embedding backend logic in graph contracts.
