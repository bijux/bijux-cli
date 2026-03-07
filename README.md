# bijux-dag

**Git for computation graphs.**

Canonical mission statements live in
`docs/spec/MISSION_STATEMENT.md`. Root docs and command help must use that wording.

## What bijux-dag is

- A deterministic computation-graph engine.
- A canonical graph and run identity model.
- A run-directory and artifact truth system with replay, diff, and proof surfaces.
- A CLI for validate/plan/run/replay/diff/inspect workflows.

## What bijux-dag is not

- Not a managed workflow platform.
- Not a scheduler control plane for distributed clusters.
- Not a claim of production container/Kubernetes/HPC execution in this repo.
- Not a replacement for every orchestration product category.

See `docs/reference/POSITIONING_NOTE.md`.

## Architecture overview

```text
                +-----------------------------+
                |         bijux-dag-cli       |
                |      operator command UX    |
                +--------------+--------------+
                               |
                               v
                +-----------------------------+
                |         bijux-dag-app       |
                | command routing + renderers |
                +--------------+--------------+
                               |
                               v
        +-----------------------------------------------+
        |                  bijux-dag-runtime            |
        | planner -> scheduler -> execution -> artifacts|
        +----------------------+------------------------+
                               |
                               v
                +-----------------------------+
                |         bijux-dag-core      |
                | canonical graph/run identity|
                +-----------------------------+

Evidence and trust boundaries:
evidence/ + bijux-dev-dag verify suites
```

## Current implemented capabilities

Short summary:
- Deterministic local DAG validation, planning, execution, replay, and diff.
- Run-directory artifact integrity verification and import/export compatibility checks.
- Evidence-governed trust reporting for release blocking vs advisory proof.

Canonical capability list:
- `docs/spec/CURRENT_IMPLEMENTED_CAPABILITIES.md`

Status matrix (implemented vs modeled/experimental/simulated):
- `ROOT_CAPABILITY_MATRIX.md`

## Quickstart

```bash
make test
make lint
make security

cargo build -p bijux-dag-cli
cargo run -p bijux-dag-cli -- dag validate evidence/authoring/examples/hello.dag.json
cargo run -p bijux-dag-cli -- dag plan evidence/authoring/examples/hello.dag.json
cargo run -p bijux-dag-cli -- dag run evidence/authoring/examples/hello.dag.json --out runs/
cargo run -p bijux-dag-cli -- dag inspect runs/run-<id>
cargo run -p bijux-dag-cli -- dag replay runs/run-<id> --out runs/
cargo run -p bijux-dag-cli -- dag diff runs/run-<id-a> runs/run-<id-b>
cargo run -p bijux-dag-cli -- dag cache verify
```

## Key references

- Mission: `docs/spec/MISSION_STATEMENT.md`
- Positioning: `docs/reference/POSITIONING_NOTE.md`
- Git mapping: `docs/reference/GIT_FOR_COMPUTATION_GRAPHS_MAPPING.md`
- Support policy: `docs/reference/EXECUTION_SUPPORT_POLICY.md`
- Glossary: `docs/reference/COMPUTATION_GRAPH_GLOSSARY.md`
- Evidence model: `docs/spec/EVIDENCE_MODEL.md`

## License

Apache-2.0. See `LICENSE` and `NOTICE`.

## Security

`make security` runs `cargo audit`. Install once:

```bash
cargo install cargo-audit
```
