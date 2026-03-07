# Cache Correctness Coverage

## Covered surfaces
- cache verify command proof checks
- cache explain key eligibility diagnostics
- cache stats invalid entry counting
- cache diff entry comparison and validity report
- cache prune simulation invalid-entry candidate listing
- warm/cold semantic fixture scenarios
- runtime cache identity and invalidation unit tests

## Coverage separation
Cache correctness coverage is tracked separately from generic e2e coverage.

## Governance rule
Cache features cannot expand faster than cache proof verification coverage.
