# As-Underscore Import Audit

## Summary

- Total `use ... as _;` imports: 2114
- Non-test/non-bench imports: 36
- Noise removals in this pass: 0 (all current uses are classified as dependency-touch or target-root exceptions)

## Count By Crate

| Crate | Count |
| --- | ---: |
| bijux-dag-app | 466 |
| bijux-dag-artifacts | 28 |
| bijux-dag-cli | 16 |
| bijux-dag-core | 120 |
| bijux-dag-runtime | 869 |
| bijux-dag-testkit | 5 |
| bijux-dev-dag | 610 |

## Classification

- Necessary dependency-touch imports in tests and benches: 2078
- Necessary dependency-touch imports in crate root entrypoints: 36
- Necessary trait reachability imports in internal modules: 0
- Noise imports: 0

## Explicit Exceptions

- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dev-dag/src/main.rs`

These exceptions remain intentional for strict target-level dependency accounting.
