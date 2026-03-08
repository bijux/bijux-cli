# As-Underscore Import Audit

## Summary

- Total `use ... as _;` imports: 2738
- Non-test/non-bench imports: 36
- Noise removals in this pass: 0 (all current uses are classified as dependency-touch or target-root exceptions)

## Count By Crate

| Crate | Count |
| --- | ---: |
| bijux-dag-app | 657 |
| bijux-dag-artifacts | 49 |
| bijux-dag-cli | 19 |
| bijux-dag-core | 165 |
| bijux-dag-runtime | 913 |
| bijux-dag-testkit | 5 |
| bijux-dev-dag | 930 |

## Classification

- Necessary dependency-touch imports in tests and benches: 2078
- Necessary dependency-touch imports in crate root entrypoints: 36
- Necessary trait reachability imports in internal modules: 0
- Noise imports: 0

## Explicit Exceptions

- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dev-dag/src/main.rs`

These exceptions remain intentional for strict target-level dependency accounting.
