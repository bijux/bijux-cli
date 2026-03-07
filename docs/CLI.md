# CLI

Bijux provides an umbrella CLI `bijux` with sub-apps. The DAG app is available as `bijux dag`.

## Commands

```
bijux dag validate <dag>
bijux dag run <dag> --out <runs/>
bijux dag replay <run-dir> --out <runs/>
bijux dag prove <run-dir>
bijux dag proof-summary <run-dir>
bijux dag diff <runA> <runB>
bijux dag explain <run-dir> [--node <id>]
bijux dag show-effective-plan <dag>
bijux dag node <run-dir> --id <id>
bijux dag status <run-dir>
bijux dag verify <run-dir> [--deep]
bijux dag fsck <run-dir> [--strict]
bijux dag fsck <bundle.json> --json
bijux dag hash run <run-dir>
bijux dag hash artifact <file>
bijux dag capabilities --json
bijux dag capabilities --backend kubernetes --json
bijux dag capabilities --backend hpc --json
bijux dag capabilities --backend remote --json
bijux dag semantic-portability --backend kubernetes --json
bijux dag equivalence-proof ./runs/a ./runs/b --backend-a kubernetes --backend-b hpc --json
bijux dag version-inspect --dag ./graph.dag.json --json
bijux dag migrate dag ./graph.dag.json --from 0.1 --to 0.1 --dry-run --json
bijux dag cache <ls|pack|unpack|verify|gc>
bijux dag adapters <ls|doctor>
bijux dag export <run-dir> --out bundle.json
bijux dag export --from-run <run-dir> --out bundle.json
bijux dag export <run-dir> --out bundle.json --without-artifacts
bijux dag export <run-dir> --out bundle.json --provenance-only
bijux dag export <run-dir> --out bundle.json --redact
bijux dag import <bundle.json>
bijux dag import <bundle.json> --verify-only
bijux completions --shell zsh
```

`dag verify --deep` performs full artifact integrity checks including path normalization,
index ordering, and schema-parse verification for stored manifest and trace files.

Command categories and long-term command support decisions are documented in:

- `docs/CLI_COMMAND_TAXONOMY.md`
- `docs/spec/CLI_BACKWARD_COMPATIBILITY.md`

## JSON Envelope

All commands accept a global `--json` flag. JSON output is normalized as:

```
{
  "ok": true,
  "command": "dag.validate",
  "data": { ... },
  "diagnostics": [ ... ]
}
```

`diagnostics` is used for validation/lint warnings or errors. Other commands return an empty array.

## Note

`bijux` is the only supported CLI entrypoint. Use `bijux dag ...` for DAG operations.

## Exit code matrix

| Command surface | Command | Success | Known failure |
| --- | --- | --- | --- |
| DAG umbrella | `dag validate` | 0 | 2 |
| DAG umbrella | `dag run` | 0 | 3 |
| DAG umbrella | `dag replay` | 0 | 3 |
| DAG umbrella | `dag diff` | 0 | 3 |
| DAG umbrella | `dag explain` | 0 | 3 |
| DAG umbrella | `dag status` | 0 | 3 |
| DAG umbrella | `dag cache` | 0 | 3 |
| DAG umbrella | `dag adapters` | 0 | 3 |
| DAG umbrella | `dag fsck` | 0 | 3 |
| DAG umbrella | `dag hash run` | 0 | 3 |
| DAG umbrella | `dag hash artifact` | 0 | 3 |
| DAG umbrella | `dag capabilities` | 0 | 0 |
| Top-level | `completions --shell <shell>` | 0 | 2 |

Failure codes are stable for CLI parser and command validation errors unless a subcommand
explicitly returns a richer status.
