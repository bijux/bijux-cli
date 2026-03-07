# Replay Speed Baseline

Replay latency baseline is tracked for three DAG sizes:

- small: `10` nodes
- medium: `100` nodes
- large: `500` nodes

Measurement method:

- run once to produce source run directory
- replay with equivalent config and cache mode
- record wall-clock duration for replay execution only

This report is updated alongside replay fidelity changes so performance regressions are visible with proof-surface changes.

