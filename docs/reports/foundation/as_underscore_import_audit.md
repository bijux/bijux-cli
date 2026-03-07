# As-Underscore Import Audit

## Summary

- Total `use ... as _;` imports: 1677
- Non-test/non-bench imports: 36
- Noise removals in this pass: 0 (all current uses are classified as dependency-touch or target-root exceptions)

## Count By Crate

| Crate | Count |
| --- | ---: |
| bijux-dag-app | 347 |
| bijux-dag-artifacts | 19 |
| bijux-dag-cli | 12 |
| bijux-dag-core | 106 |
| bijux-dag-runtime | 858 |
| bijux-dev-dag | 335 |

## Classification

- Necessary dependency-touch imports in tests and benches: 1641
- Necessary dependency-touch imports in crate root entrypoints: 36
- Necessary trait reachability imports in internal modules: 0
- Noise imports: 0

## Explicit Exceptions

- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dev-dag/src/main.rs`

These exceptions remain intentional for strict target-level dependency accounting.
