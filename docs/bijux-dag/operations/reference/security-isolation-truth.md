---
title: Security And Isolation Truth
audience: operators
type: reference
status: canonical
owner: bijux-dag-docs
last_reviewed: 2026-07-08
---

# Security And Isolation Truth

This page states the actual execution-boundary guarantees for `bijux-dag`
`v0.4.0`.

It exists to answer one question clearly:

What does the runtime really enforce, what is only best-effort, and what is not
protected at all?

The release posture is intentionally local-first and honest. `bijux-dag` is a
serious local DAG runtime, but it is not a general-purpose host sandbox, VM
boundary, network firewall, or clock virtualization layer.

## Truth table

| Surface | What is enforced | What is best-effort | What is not protected |
| --- | --- | --- | --- |
| shell backend | declared-effect policy gates for `network`, `env`, and `clock`; shaped environment; non-blank `argv[0]` validation; declared output target preflight; output-root and run-dir path validation | subprocess boundary; Unix subprocess-group cleanup on timeout or cancellation | socket-level network firewalling; clock or syscall interposition; arbitrary filesystem-read sandboxing; host side effects after spawn |
| container backend | declared-effect policy gates; engine no-network flags for supported engines; digest policy for container image references when required; constrained node mounts; declared output target preflight; shaped environment | isolation quality of the selected container runtime; engine-reported image identity | VM-grade isolation; clock virtualization; registry signature trust; full host sandboxing beyond the container runtime |
| clean environment | environment shaping through an allowlist and denylist model | correctness depends on honest allowlist declarations | filesystem, process, network, or clock isolation |
| deny-network | refuse nodes that declare the `network` effect; pass `--network none` to supported container engines | only as strong as accurate effect declarations and the chosen container engine | host-level network isolation for shell subprocesses or dishonest nodes |
| deny-clock | refuse nodes that declare the `clock` effect | only as strong as accurate effect declarations | time freezing, fake clocks, or wall-clock syscall isolation |
| filesystem boundaries | output-path authorization, storage-relative path validation, run-dir write ownership, container mount-root checks | host-process discipline for shell commands that run inside their node work tree | arbitrary host reads by shell code; full host write prevention outside the governed output and storage helpers |
| replay `--sandbox` | forbids writes into the source run directory | safe evidence reuse for inspection when retained evidence is complete | process sandboxing, network isolation, clock isolation, or a general filesystem sandbox |

## Shell backend

The shell backend is a host-process execution model with policy gates in front
of it.

### Enforced

- `deny-network` refuses nodes that declare `Effect::Network`
- `deny-env` refuses nodes that declare `Effect::Env`
- `deny-clock` refuses nodes that declare `Effect::Clock`
- shell `argv` must be a non-empty array of strings and `argv[0]` must be a
  non-blank executable token before the runtime attempts process launch
- `clean-env` shapes the launched environment through the effective allowlist
- missing required exact environment bindings fail before execution starts
- declared output targets are authorized before launch, so paths such as
  `../escape.txt` never become writable targets
- symlinked existing parent components in declared output paths are rejected
  before the shell subprocess starts
- undeclared or missing outputs fail finalization
- retained output and run-directory paths stay rooted under governed storage

### Best-effort

- shell execution gets a subprocess boundary rather than in-process execution
- on Unix hosts, timed-out or cancelled subprocesses are terminated as a
  process group so background descendants do not keep running after node exit

### Not protected

- there is no socket firewall for shell subprocesses
- there is no wall-clock or time-syscall virtualization
- there is no arbitrary host filesystem-read sandbox
- the runtime does not interpose on syscalls after the executor has started
- a node that lies about its effects still runs as host code unless another
  boundary blocks it

## Container backend

The container backend is stronger than the shell backend for mount shaping and
engine-managed no-network mode, but the claim stops at the container runtime
boundary.

### Enforced

- the runtime validates the container contract before launch
- built-in container execution supports `docker` and `podman`
- `deny-network` passes container runtime no-network flags and fails closed
  when the built-in adapter cannot honor them
- input, output, and work mounts are validated against the node root
- inputs are mounted read-only
- outputs and work are mounted writable
- declared output targets are authorized before the container starts
- declared output paths must stay normalized and relative
- traversal such as `../escape.txt` and symlinked existing parent components
  are rejected before a writable output target is handed to the adapter
- digest-pinned image references can be required by policy before execution

### Best-effort

- isolation strength depends on the selected container engine and host runtime
- image identity evidence still depends on what the engine reports back

### Not protected

- this is not a VM boundary
- the runtime does not claim registry signature verification or publisher trust
- clock denial is still declaration-based rather than syscall-based
- full host isolation is not claimed beyond the container runtime boundary

## Clean environment

`clean-env` is environment shaping, not process isolation.

### Enforced

- the runtime computes an effective allowlist from node and container
  declarations
- exact required bindings that are absent from the ambient environment fail
  before execution
- denylist matches are dropped
- when `clean_env` is enabled, the launched environment starts from a stripped
  view and only permitted bindings are reintroduced

### Best-effort

- environment discipline is only as precise as the graph’s declared allowlist

### Not protected

- `clean-env` does not sandbox filesystem access
- `clean-env` does not prevent host-visible side effects
- `clean-env` does not stop a process from using network or time syscalls by
  itself

## Network policy

`deny-network` is implemented in two different ways depending on the executor.

### Enforced

- all executors refuse nodes that declare network effects when policy denies
  them
- supported container engines receive explicit no-network flags

### Best-effort

- the shell backend depends entirely on honest effect declarations
- container no-network semantics depend on the selected engine

### Not protected

- shell subprocesses do not get a runtime network firewall
- dishonest or incomplete effect declarations can weaken the practical outcome

## Clock policy

`deny-clock` is a declaration gate, not a time sandbox.

### Enforced

- the runtime refuses nodes that declare clock effects when policy denies them

### Best-effort

- the guarantee depends on honest effect declarations in the DAG

### Not protected

- there is no fake clock
- there is no frozen wall clock
- there is no syscall interception for time access

## Filesystem boundaries

Filesystem safety in `bijux-dag` is about rooted writes and validated paths,
not about complete host sandboxing.

### Enforced

- storage-relative paths reject traversal, absolute paths, and backslash escapes
- run outputs, cache keys, and governed storage writes go through owned storage
  helpers
- declared output targets are preflight-authorized before adapter execution
- input and output authorization rejects paths that escape the rooted input or
  output tree
- malicious declared output paths such as `../x` or symlinked parent escapes
  are rejected before the runtime hands out a write path
- container mounts are restricted to validated paths beneath the node root
- replay sandbox mode forbids writing into the source run directory

### Best-effort

- shell commands still run as host processes that are expected to respect the
  node work, input, and output layout

### Not protected

- shell code can still attempt arbitrary host reads
- the runtime does not claim a general syscall sandbox for file access
- host writes outside governed helper paths are not prevented by a kernel-level
  sandbox

## What to trust

Trust these as current `v0.4.0` guarantees:

- fail-closed policy denial for honestly declared `network`, `env`, and `clock`
  effects
- deterministic environment shaping and exact required-env preflight failure
- rooted run-dir, output, and cache-storage path validation
- stronger mount and no-network controls for supported container engines
- source-run write protection for replay sandbox mode
- Unix subprocess-group cleanup for timeout and cancellation

Do not trust these as current `v0.4.0` guarantees:

- shell sandboxing
- shell network firewalling
- shell filesystem sandboxing
- clock virtualization
- VM-grade container isolation
- replay process sandboxing
- registry signature or publisher-trust enforcement

## Code anchors

- `crates/bijux-dag-runtime/src/internal/control/runtime_controls.rs`
- `crates/bijux-dag-runtime/src/internal/identity/security_env.rs`
- `crates/bijux-dag-runtime/src/artifacts/storage/path_authorization.rs`
- `crates/bijux-dag-runtime/src/artifacts/storage/store.rs`
- `crates/bijux-dag-runtime/src/backend/runtime/container_execution.rs`
- `crates/bijux-dag-runtime/tests/policy_cache_contract.rs`
- `crates/bijux-dag-runtime/tests/security_model_contracts.rs`
- `crates/bijux-dag-runtime/tests/subprocess_cleanup_contracts.rs`
- `crates/bijux-dag-app/tests/policy_enforcement_surface_contract.rs`

## Related references

- [Security and Safety](../security-and-safety.md)
- [Trust Boundaries](trust-boundaries.md)
- [Known Limitations](../../quality/known-limitations.md)
