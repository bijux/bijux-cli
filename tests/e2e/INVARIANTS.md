# E2E Invariants

These are required for every E2E test case.

1) CLI never crashes (no Python traceback in output).
2) State transitions are reversible where promised.
3) Idempotent commands remain idempotent across repeats.
4) Invalid input never corrupts state.
5) Plugin lifecycle leaves no residue on disk.
6) Exit codes are stable and consistent.
