# bijux-dag

Most pipeline stacks can tell you whether a run passed, but they struggle to prove what changed across runs and whether the change matters. `bijux-dag` exists to make graph execution evidence auditable: stable graph identity, attempt-level run identity, artifact lineage, replay classification, and semantic diff.

`bijux-dag` is Git for computation graphs in one practical sense: you can identify, compare, and reason about graph execution history with explicit contracts instead of ad hoc logs.

Unlike orchestration-first systems, `bijux-dag` centers evidence interpretation before platform control features. Unlike build systems, it models full graph-run-artifact history and replay/diff semantics as first-class objects.

## Why this exists

Teams need answers that survive incidents and handoffs:
- what exact graph definition ran,
- what outputs were produced by which node,
- whether a replay stayed equivalent or drifted,
- where semantic change entered the system.

`bijux-dag` is designed for those questions.

## What bijux-dag is not

- Not a managed orchestration service.
- Not a guarantee that every backend behaves equivalently.
- Not a replacement for operator trust-boundary checks.

## Core object model

- `graph`: canonical computation definition and dependency topology.
- `run`: one concrete execution attempt of a graph.
- `artifact`: produced output linked to producing run and node.
- `replay`: evidence-oriented re-execution classified as `equivalent`, `drift`, or `incomplete`.
- `diff`: semantic comparison for graph/run/artifact surfaces.

```text
graph -> run -> artifacts -> replay/diff -> release or block
```

## First 2 minutes

Use the end-user CLI surface (`bijux-dag-cli`) for normal usage.

Create `examples/hello.dag.json`:

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

Run it end to end:

```bash
cargo run -p bijux-dag-cli -- dag validate examples/hello.dag.json
cargo run -p bijux-dag-cli -- dag run examples/hello.dag.json --out runs/
cargo run -p bijux-dag-cli -- dag inspect runs/run-<id>
cargo run -p bijux-dag-cli -- dag replay runs/run-<id> --out runs/
cargo run -p bijux-dag-cli -- dag diff runs/run-<baseline-id> runs/run-<candidate-id>
```

Expected outcomes:
- `out/message.txt` and `out/message.count` are produced,
- run directory evidence is created under `runs/`,
- replay/diff can classify equivalence or drift using run evidence.

## Typical workflow

```bash
cargo run -p bijux-dag-cli -- dag validate <graph>
cargo run -p bijux-dag-cli -- dag run <graph> --out runs/
cargo run -p bijux-dag-cli -- dag inspect runs/run-<id>
cargo run -p bijux-dag-cli -- dag replay runs/run-<id> --out runs/
cargo run -p bijux-dag-cli -- dag diff runs/run-<baseline-id> runs/run-<candidate-id>
```

## CLI paths

- User path: `cargo run -p bijux-dag-cli -- ...`
- Maintainer path: `cargo run -p bijux-dev-dag -- ...` (repository guardrails, audits, and governance checks)

## License

Apache-2.0. See `LICENSE`.
