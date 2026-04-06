# Release Evidence Strength Before And After

Date: 2026-03-07

## Before

- Release messaging leaned on aggregate test pass totals.
- Blocking vs advisory evidence was weakly separated.
- Some proof surfaces were present but not forced into release verification.

## After

- Release verification enforces explicit blocking and advisory sets.
- Claimed proof surfaces (`replay`, `cache`, `operator`) are validated.
- Drift checks ensure manifests and reports remain synchronized with owned evidence.

## Strength indicators

| Indicator | Before | After |
| --- | ---: | ---: |
| explicit blocking asset set | partial | 7 enforced |
| explicit advisory asset set | partial | 2 enforced |
| claimed proof surface validation | no | yes |
| release drift checks | weak | strict |
| release reports generated | no | yes (`what_this_release_proves`, `what_this_release_does_not_prove`, `unsupported_or_simulated_areas`) |

## Remaining weakness

- Blocking set breadth in `release_evidence_set.json` is intentionally small and should be expanded only with high-trust battle scenarios, not metadata-heavy additions.
