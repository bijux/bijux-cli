# Execution And Scheduling

The controller is authoritative for node readiness, attempt lifecycle, and
terminal run state. Backend status is evidence supplied to that controller,
not an independent state machine.

## Admission

Execution starts from a validated plan. Admission resolves runtime policy,
backend capabilities, selectors, resource requirements, adapter availability,
security constraints, and retained run context.

An unsupported node kind, resource, execution mode, or policy combination is a
refusal before launch. Admission does not silently substitute a different
backend.

## Readiness And Ordering

A node becomes eligible only when dependencies have terminal outcomes and its
trigger rule evaluates true. Branch decisions prune unselected lanes and
record skipped outcomes. Scheduler ordering is deterministic for equivalent
ready sets and policy.

Concurrency limits affect when ready nodes run, not graph meaning. Resource
and fairness policy must be explicit inputs and appear in diagnostics.

## Attempt Lifecycle

Each attempt records:

- node, run, and attempt identity;
- resolved execution context and backend/adapter identity;
- start and terminal timestamps from the runtime clock;
- stdout, stderr, exit, timeout, and cancellation evidence;
- required-output and persistence results;
- failure class and retry decision.

Retries append attempts. They never overwrite the original failure. Retry
policy is evaluated against the normalized failure class and attempt count.

## Cancellation And Timeout

Cancellation is checked before launch, during supported execution, and before
accepting terminal success. Timeout and cancellation remain distinct. Unix
subprocesses use process-group cleanup; other hosts report their explicit
best-effort guarantee.

A late successful process exit does not supersede an accepted timeout or
cancellation decision.

## State Invariants

- A node has one accepted terminal state.
- Every transition is legal from the current state and recorded once.
- Success requires all required outputs and evidence writes.
- Skipped nodes include trigger or branch reasoning.
- Run completion agrees with terminal node counts.
- Causal failure survives retries, repair, and resume.
- Secret values do not enter logs, traces, cache identity, or commands.

## Recovery Boundary

Local run resume and evidence recovery are explicit. Kubernetes and SLURM
controller-restart recovery is not part of the stable promise. Recovery cannot
invent missing attempts or accept ambiguous staging/final run state.

## Verification

Principal authorities include:

```bash
cargo test --locked -p bijux-dag-runtime --test runtime_state_machine_contracts
cargo test --locked -p bijux-dag-runtime --test runtime_scheduler_contracts
cargo test --locked -p bijux-dag-runtime --test runtime_retry_contracts
cargo test --locked -p bijux-dag-runtime --test runtime_cancellation_contracts
```

Determinism, adversarial, resilience, and formal-invariant contracts cover
ordering and failure combinations beyond representative examples.
