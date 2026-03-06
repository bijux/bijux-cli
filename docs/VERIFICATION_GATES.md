# Verification gates

## Mandatory gates

- invariant suite gate
- property-based suite gate
- model-based suite gate

A release passes only when all required gates are green.

## Gate evidence

Each gate run must publish:
- verification summary
- failing invariants (if any)
- counterexample reports
- maturity label deltas
