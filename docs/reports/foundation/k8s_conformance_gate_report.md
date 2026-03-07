# Kubernetes Conformance Gate Report

## Release gate

Kubernetes support claims are blocked unless all Kubernetes fixture contracts pass.

## Required fixtures

- `evidence/battle/fixtures/kubernetes/tiny_equivalence.dag.json`
- `evidence/battle/fixtures/kubernetes/medium_fanout.dag.json`
- `evidence/battle/fixtures/kubernetes/failure_injection_image_pull_backoff.dag.json`
- `evidence/battle/fixtures/kubernetes/k8s_vs_local_run_diff.json`
- `evidence/operator/fixtures/kubernetes_pod_failure_explain.json`

## Required portability and replay checks

- Kubernetes replay-from-import conformance simulation must pass.
- Kubernetes-origin bundle export/import summary must preserve provenance source.
- Kubernetes-vs-local diff fixture must remain semantically equivalent.

## Required contract suites

- `crates/bijux-dag-runtime/tests/backend_cluster_contracts.rs`
- `crates/bijux-dev-dag/tests/k8s_adapter_contracts.rs`
- `crates/bijux-dev-dag/tests/k8s_adapter_release_contracts.rs`
