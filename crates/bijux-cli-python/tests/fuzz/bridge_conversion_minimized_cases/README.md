# Bridge Conversion Minimized Cases

These JSON files are regression inputs for
`tests/bridge_conversion_case_replays.rs`. The suite reads every `.json` file
in lexical order and verifies that repeated `serde_json` conversion gives the
same success or failure classification.

The current corpus contains one success envelope and one error envelope. It
keeps both top-level result shapes represented without duplicating the broader
bridge contract fixtures.

## Replay

From the repository root:

```sh
cargo test -p bijux-cli-python --test bridge_conversion_case_replays
```

## Scope

This replay detects non-deterministic JSON acceptance and accidental loss of
the retained envelope shapes. It does not prove Python object conversion,
exception mapping, or Rust/Python command parity; those behaviors are covered
by the adjacent bridge conversion and parity suites.

## Updating The Corpus

- Retain one JSON value per file with a `.json` extension.
- Reduce a failing input to the smallest payload that still reproduces the
  behavior.
- Name the case by stable behavior, not discovery order, when adding new files.
- Explain in the change why existing bridge contract fixtures did not cover
  the regression.
- Run this replay and the test that originally exposed the defect.

Generated fuzz output and reduction logs belong under `artifacts/`, not here.
