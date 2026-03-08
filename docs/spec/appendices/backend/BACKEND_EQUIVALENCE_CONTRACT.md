# Backend Equivalence Contract

## Purpose
Define backend equivalence semantics and downgrade behavior across local, kubernetes, hpc, and remote targets.

## Equivalence classes
- `equivalent`: run outcomes and artifact lineage are semantically equivalent.
- `fidelity-preserving`: semantics preserved with advisory differences that do not change graph meaning.
- `downgraded`: semantics cannot be preserved fully; downgrade must be explicit.

## Required behavior
- Unsupported backend semantics must be rejected, not approximated.
- Backend-specific metadata must not alter graph identity.
- Backend-specific environment/runtime metadata may affect run identity where declared.
- Cross-backend replay and diff must emit explicit downgrade reasons.

## Operator surfaces
- `bijux dag capabilities --backend <name> --json`
- `bijux dag semantic-portability --backend <name> --json`
- `bijux dag equivalence-proof <run-a> <run-b> --backend-a <name> --backend-b <name> --json`
