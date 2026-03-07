# Developer control-plane architecture

`bijux-dev-dag` is organized as:

- `cli`: command-line integration boundary
- `commands`: command dispatch and workflow execution
- `suites`: typed suite metadata and release verification flow
- `report`: machine-readable command report schema and writers
- `policy`: governance policy locations and loaders
- `tooling`: subprocess boundaries (`cargo`, `git`) and command-runner abstraction
- `repo`: workspace root and required layout contracts

## Release verify composition

`release verify` is a composed flow of typed suite groups:

1. `checks.run`
2. `tests.run`
3. `contracts.run`
4. `docs.run`
5. `repo.run`

The composed flow is declared in `crates/bijux-dev-dag/src/suites/release.rs`.
