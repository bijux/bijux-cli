# CLI

## Audience and ownership

Audience: operators and maintainers.
Owner: CLI platform team.
Status: stable.

## Purpose

Single operator entrypoint for running, validating, replaying, diffing, and inspecting DAG workflows.

## CLI surface

```bash
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
bijux dag cache <ls|pack|unpack|verify|gc>
bijux dag adapters <ls|doctor>
bijux dag export <run-dir> --out bundle.json
bijux dag import <bundle.json> [--verify-only]
bijux completion --shell zsh
```

## Command categories and stability model

- Product commands: `validate`, `run`, `replay`, `diff`, `explain`, `status`, `cache`, `verify`, `hash`, `capabilities`, `adapters`, `export`, `import`.
- Debug commands are available on `dag` and should be treated as diagnostics.
- Top-level utility: `completions`.

### Compatibility and lifecycle

- Stable JSON envelope: `ok`, `command`, `data`, `diagnostics`.
- Stable command names and legacy aliases are governed by compatibility and deprecation contracts.
- New aliases require explicit migration notes and contract tests.

For full contract text, see:

- [CLI backward compatibility contract](./spec/CLI_BACKWARD_COMPATIBILITY.md)
- [CLI ownership boundaries](./spec/CLI_OWNERSHIP.md)
- [CLI deprecation and alias policy](./spec/CLI_DEPRECATION_AND_ALIAS_POLICY.md)
- [CLI command taxonomy](./reference/COMMAND_TAXONOMY.md)

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

## Backward-compatibility posture

- Command classes documented in this guide are part of the stable CLI surface unless explicitly marked experimental.
- Legacy diagnostics aliases should preserve status classes and schema class while migration is in effect.
- Alias and deprecation behavior follows the canonical contracts above.
