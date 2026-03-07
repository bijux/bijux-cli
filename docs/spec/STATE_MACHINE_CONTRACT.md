# State machine contract

## Scope

Defines formal node and run state machines, legal transitions, invariant IDs, and consistency verification behavior.

## Node states

- pending
- eligible
- queued
- running
- success
- failed
- skipped
- cached
- cancelled

## Run states

- submitted
- planning
- running
- paused
- interrupted
- cancelling
- cancelled
- failed
- succeeded

## Invariant IDs

- node transition invariants: `INV-NODE-TRANSITION-*`
- node terminal no-revert invariant: `INV-NODE-TERMINAL-REVERT-001`
- run transition invariants: `INV-RUN-TRANSITION-*`
- failed run causal invariant: `INV-RUN-FAILED-CAUSAL-001`

## Transition guards

Illegal transitions must fail loudly in debug and test paths via transition validation functions.

## Post-run consistency checks

`verify_post_run_state_consistency` validates:

- terminal run has terminal node states
- cancelled run includes cancelled nodes
- failed run contains at least one causal failure

## Operator inspection

`bijux-dev-dag dag verify-state --run-dir <path>` checks state coherence from run artifacts.
