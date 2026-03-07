# Operator inspection guide

## Which command answers which question

- `dag runs show`: concise run summary for status, integrity, and timing.
- `dag runs inspect`: richer run summary including retries, cache hits, and artifact count.
- `dag runs tree`: graph-structured node status view.
- `dag runs timeline`: execution lifecycle ordering with retry/cache markers.
- `dag runs explain-failure`: root failure and propagated/skip grouping.
- `dag runs verify`: run validity checks against required artifacts.
- `dag runs doctor`: corruption diagnosis for broken run directories.

## Typical investigation flow

1. Start with `dag runs show`.
2. If suspicious, run `dag runs inspect`.
3. Use `dag runs timeline` and `dag runs tree` to localize behavior.
4. Use `dag runs explain-failure` for causal failure grouping.
5. Finish with `dag runs verify` and `dag runs doctor` when integrity is uncertain.
