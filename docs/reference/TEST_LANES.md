# Test lanes

## Fast lane

- `make test`
- Executes `cargo run -p bijux-dev-dag -- tests run`
- Intended for local iteration and pull request feedback.

## Full lane

- `make test-all`
- Executes `evidence-battle` and `evidence-consumers`.
- Intended for release-significant checks and deeper governance coverage.

## App tests currently skipped in fast lane

The following app tests are marked `#[ignore]` and are not part of fast default runs:

- `crates/bijux-dag-app/tests/error_output_contract.rs::json_error_output_contains_structured_fields`

Any new ignored tests must be documented here with a short justification.
