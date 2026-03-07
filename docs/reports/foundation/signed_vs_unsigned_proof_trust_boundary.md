# Signed vs Unsigned Proof Trust Boundary

## Unsigned proof

- carries structural and semantic evidence
- does not provide cryptographic origin authenticity

## Signed proof (future hook)

- would include signature format and signature payload
- requires verification key distribution and trust policy

## Current contract

Proof output includes signing metadata hooks and currently reports `trust_level: unsigned`.
