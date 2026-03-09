# Kubernetes Adapter Benchmark Report

Status: contract benchmark baseline for simulated adapter surfaces.

## Startup latency benchmark

- Scenario: `k8s-startup-latency`
- Measurement focus: adapter startup and mapping preparation overhead.
- Current status: contract baseline defined; production benchmark blocked until implemented backend path.

## Many-small-node benchmark

- Scenario: `k8s-many-small-node-dag`
- Measurement focus: dispatch and terminal-event reduction overhead under high node counts.
- Current status: contract baseline defined; production benchmark blocked until implemented backend path.

## Large-artifact benchmark

- Scenario: `k8s-large-artifact-dag`
- Measurement focus: artifact collection and state classification overhead.
- Current status: contract baseline defined; production benchmark blocked until implemented backend path.

## Notes

All benchmark claims must remain simulation-labeled until Kubernetes execution mode moves from simulated to implemented in support policy.
