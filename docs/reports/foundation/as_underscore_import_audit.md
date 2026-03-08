# As-Underscore Import Audit

## Summary

- Total `use ... as _;` imports: 3200
- Non-test/non-bench imports: 91
- Noise removals in this pass: 0 (all current uses are classified as dependency-touch or target-root exceptions)

## Count By Crate

| Crate | Count |
| --- | ---: |
| bijux-dag-app | 742 |
| bijux-dag-artifacts | 58 |
| bijux-dag-cli | 19 |
| bijux-dag-core | 179 |
| bijux-dag-runtime | 933 |
| bijux-dag-testkit | 5 |
| bijux-dev-dag | 1264 |

## Classification

- Necessary dependency-touch imports in tests and benches: 3109
- Necessary dependency-touch imports in crate root entrypoints: 91
- Necessary trait reachability imports in internal modules: 0
- Noise imports: 0

## Explicit Exceptions

- `crates/bijux-dag-app/src/lib.rs`
- `crates/bijux-dev-dag/src/main.rs`

These exceptions remain intentional for strict target-level dependency accounting.
