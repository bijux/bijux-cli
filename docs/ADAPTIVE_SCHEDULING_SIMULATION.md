# Adaptive scheduling simulation harness

## Scenario families

- congestion with SLA-critical queues
- retry storms with unstable backends
- artifact-store pressure events
- replay-heavy windows with prefetch hints

## Evaluation outputs

- static vs adaptive dispatch latency
- static vs adaptive SLA miss rate
- adaptive drift signal against baseline
- fallback trigger decision

## Acceptance expectation

Adaptive simulation is accepted only when no scenario violates semantic invariants or policy bounds.
