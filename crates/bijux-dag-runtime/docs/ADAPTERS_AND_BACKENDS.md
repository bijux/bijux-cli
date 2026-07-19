# Adapters And Backends

Adapters define how a node operation is invoked. Backends define where and
under what execution capabilities an attempt runs. The runtime normalizes both
into one controller-owned result model.

## Adapter Contract

An adapter declares identity, supported node/effect behavior, cache
compatibility, and execution entrypoints. `AdapterContext` supplies explicit
paths, params, environment policy, cancellation, and run identity.

Built-in adapters cover constant values, shell, containers, and file
transforms. Python, HTTP, and external adapters cross additional trust or
network boundaries and require explicit policy.

External adapters use a versioned descriptor handshake. Probe failure,
incompatible protocol, unavailable executable, launch failure, invalid result,
and adapter-reported failure remain distinct outcomes. Adapter binary identity
participates in cache evidence.

## Backend Contract

A backend advertises capabilities and accepts a normalized execution request.
The runtime records backend kind, configuration, and relevant implementation
identity. Backend-private statuses map to runtime-owned lifecycle results
without losing diagnostic context.

| Backend | Stable scope | Required boundary |
| --- | --- | --- |
| local | shell and local-container attempts | host process or configured container engine |
| Kubernetes | container nodes submitted as Jobs | `kubectl`, shared PVC, host/cluster path mapping |
| SLURM | configured worker command | `sbatch`, `sacct`, shared run directory |

Local execution is not VM isolation. Kubernetes support is not a workflow
controller. SLURM support is not a generic HPC abstraction.

## Containers

Container execution validates engine discovery, image reference policy,
network and GPU options, mounted paths, and relative output declarations.
Host-to-container mappings preserve the run layout. Engine and resolved image
identity are retained for replay and cache decisions.

## Batch Execution

Kubernetes maps container, resource, timeout, and shared-volume requirements
to a Job, then normalizes pod/job state and logs. SLURM maps resources and
walltime to submission options, polls terminal state, and collects results
from shared storage.

Duplicate status delivery, stale heartbeats, cancellation, scheduler failure,
and artifact collection are explicit classifications.

## Capability Refusal

Unsupported fields or unavailable requirements are rejected before launch.
Modeled remote-worker, distributed, federation, and high-availability
contracts cannot be selected as stable execution services.

## Verification

```bash
cargo test --locked -p bijux-dag-runtime --test adapter_conformance
cargo test --locked -p bijux-dag-runtime --test execution_backend_contract
cargo test --locked -p bijux-dag-runtime --test container_execution_contracts
cargo test --locked -p bijux-dag-runtime --test kubernetes_execution_contracts
cargo test --locked -p bijux-dag-runtime --test slurm_execution_contracts
```

Run subprocess cleanup and security contracts whenever launch or termination
behavior changes.
