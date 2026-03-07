# Feature development freeze policy

## Rule
No new feature surfaces are introduced until foundation readiness criteria are satisfied.

## Evidence governance linkage
All new scenario-like assets must comply with `evidence/CONTRACT.md` and be registered in `evidence/ownership/evidence_ledger.json`.
Repository proof pillars are frozen: no new top-level proof roots beyond `evidence/`.

## Allowed during freeze
- contract clarification
- governance enforcement
- correctness fixes
- migration and compatibility safety work

## Disallowed during freeze
- new product surfaces without matching foundation evidence
- speculative runtime expansion without ownership and contract mapping
- scenario-like files added outside evidence-governed roots

## Lift condition
Freeze is lifted only when the foundation final report confirms readiness criteria satisfaction.
