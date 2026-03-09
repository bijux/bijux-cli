# bijux-dag

`bijux-dag` helps you run computation graphs with evidence you can trust later: stable graph identity, attempt-level run history, artifact lineage, replay validation, and semantic diff.

Most pipeline tools tell you what happened *now*. They are weaker at proving what changed, why it changed, and whether the change is meaningful. `bijux-dag` exists to close that gap with explicit identity and comparison contracts.

In one concrete sentence: `bijux-dag` is like Git for computation graphs because it gives stable identity, history, and comparison primitives for graph execution evidence, not just one-off job runs.

## Why this exists

Teams need more than "green" or "red" jobs. They need auditable answers to:
- what graph ran,
- what outputs were produced,
- whether a replay stayed equivalent,
- where semantic drift started.

`bijux-dag` is built around those questions.

## What bijux-dag is not

- It is not a managed orchestration platform.
- It is not a universal backend-equivalence claim.
- It is not a replacement for trust-boundary verification in operations.

## Core ideas

- `graph`: the canonical computation definition and dependency structure.
- `run`: one concrete execution attempt of a graph.
- `artifact`: output produced by a run, with lineage to producing node/run.
- `replay`: evidence-oriented re-execution to classify equivalence, drift, or incomplete.
- `diff`: semantic comparison across graph/run/artifact surfaces.

```text
graph -> run -> artifacts -> replay/diff -> decision
```

## First 5 minutes

Create a small DAG with a dependency and a visible output:

```json
{
  "version": "1",
  "graph": {
    "nodes": [
      {
        "id": "prepare",
        "command": "mkdir -p out && echo 'hello bijux' > out/message.txt"
      },
      {
        "id": "summarize",
        "depends_on": ["prepare"],
        "command": "wc -c out/message.txt > out/message.count"
      }
    ]
  }
}
```

Save as `examples/hello.dag.json`, then run:

```bash
cargo run -p bijux-dag-cli -- dag validate examples/hello.dag.json
cargo run -p bijux-dag-cli -- dag run examples/hello.dag.json --out runs/
cargo run -p bijux-dag-cli -- dag inspect runs/run-<id>
```

Continue with verification:

```bash
cargo run -p bijux-dag-cli -- dag replay runs/run-<id> --out runs/
cargo run -p bijux-dag-cli -- dag diff runs/run-<baseline-id> runs/run-<candidate-id>
```

## What you get after a run

- a run directory with execution evidence,
- artifacts such as `out/message.txt` and `out/message.count`,
- inspectable identity and lineage fields used by replay/diff.

## Typical workflow

```bash
cargo run -p bijux-dag-cli -- dag validate <graph>
cargo run -p bijux-dag-cli -- dag run <graph> --out runs/
cargo run -p bijux-dag-cli -- dag inspect runs/run-<id>
cargo run -p bijux-dag-cli -- dag replay runs/run-<id> --out runs/
cargo run -p bijux-dag-cli -- dag diff runs/run-<baseline-id> runs/run-<candidate-id>
```

## Repository contents

- user-facing guides and contracts in `docs/`
- runnable and inspectable examples in `examples/`
- runtime and CLI implementation in `crates/`
- contract guardrails and verification tooling in `crates/bijux-dev-dag`

## Start here (read next)

- Learn the model: [What is bijux-dag](docs/01-introduction/01-what-is-bijux-dag.md)
- Run your first pipeline: [Getting started](docs/02-getting-started/03-running-a-pipeline.md)
- Use commands confidently: [CLI overview](docs/04-cli-reference/01-cli-overview.md)
- Understand exact guarantees: [Specification](docs/06-specification/01-dag-model.md)

## Maintainer-oriented commands

```bash
cargo run -p bijux-dev-dag -- --help
cargo test -p bijux-dev-dag
```

## License

Apache-2.0. See `LICENSE`.
