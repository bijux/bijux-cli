# State machine hardening report

## Scope

Captures the hard guarantees for node and run state semantics used as foundation evidence.

## Canonical state machines

Node states:
- pending
- eligible
- queued
- running
- success
- failed
- skipped
- cached
- cancelled

Run states:
- submitted
- planning
- running
- paused
- interrupted
- cancelling
- cancelled
- failed
- succeeded

## Invariant identities

Mandatory invariant identities:
- `INV-NODE-TRANSITION-*`
- `INV-NODE-TERMINAL-REVERT-001`
- `INV-RUN-TRANSITION-*`
- `INV-RUN-FAILED-CAUSAL-001`

## Transition guard guarantees

- Illegal node and run transitions fail validation deterministically.
- Terminal node states do not revert to non-terminal states.
- Cached and skipped are terminal outcomes with explicit causes.
- Failed runs require causal failure evidence unless cancellation is terminal.

## Verification and operator surfaces

- Post-run verifier: `verify_post_run_state_consistency`
- Operator command: `bijux-dev-dag dag verify-state --run-dir <path>`
- Transition audit events: `terminal_transition_audit_events`

## Battle evidence linkage

State-machine conformance is mandatory evidence for battle workflows through trust property `tp_state_machine_legality` and the battle suite guard.
