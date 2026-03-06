# Makefile and developer commands

## Principle

Root `Makefile` is a thin wrapper around `bijux-dev-dag`.

## Canonical mapping

- `make test` -> `cargo run -p bijux-dev-dag -- tests run`
- `make checks` -> `cargo run -p bijux-dev-dag -- checks run`
- `make tests-all` -> `cargo run -p bijux-dev-dag -- tests run`
- `make contracts-all` -> `cargo run -p bijux-dev-dag -- contracts run`
- `make release-verify` -> `cargo run -p bijux-dev-dag -- release verify`
- `make artifacts-clean` -> `cargo run -p bijux-dev-dag -- artifacts-clean`

## Output and reporting

- `--output-paths` is declared for reproducible artifact locations.
- `make` failure messages include artifact location and report guidance.
