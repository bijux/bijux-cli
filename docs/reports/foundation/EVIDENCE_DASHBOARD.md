# Evidence Dashboard

## Release-Critical Families
- battle
- cache
- replay (under cache)
- compat
- fault
- operator
- perf
- consumers governance
- release-set governance

## Advisory Families
- compare

## Lane Summary
- fast lane: blocks on `battle` and `consumers`; advisory evidence remains non-blocking
- full lane (`make test-all`): blocks on release-critical evidence command set

## Ownership Links
- command-to-doc map: `docs/reports/foundation/archive/EVIDENCE_COMMAND_OWNER_MAP.md`
- family governance policy: `configs/policy/evidence_family_governance.json`
