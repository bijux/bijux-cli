# Minimal Consumer Fixture

This fixture represents another project consuming bijux-dag through the CLI surface.

## Usage
```sh
cargo run -p bijux-dag-cli --bin bijux-dag -- validate dag.json --strict
cargo run -p bijux-dag-cli --bin bijux-dag -- run dag.json --out runs --run-id fixture-run
```
