# DAG Configuration Contract

`configs/dag/` contains repository-owned machine-readable inputs for DAG
runtime defaults, deployment profiles, governance policy, release checks,
schedules, schemas, and maintainer suites. A file is authoritative only when a
named product or maintainer code path consumes it.

## Directory Ownership

| Directory | Responsibility |
| --- | --- |
| `deployment/` | named local, CI, and cluster deployment profiles |
| `dev/` | maintainer runtime defaults, examples, and suite overrides |
| `policy/` | governance inputs consumed by contract tests or `bijux-dev-dag` commands |
| `release/` | release checklist, smoke, artifact, and validation-suite inputs |
| `schedules/` | repository-owned schedule registry |
| `schema/` | DAG, retained evidence, control-plane, benchmark, and operator JSON schemas |
| `suites/` | declarative maintainer suite selections |

Directory placement does not make a file executable or public. The consuming
code, schema validation, and tests establish its authority.

## Change Rules

- Preserve stable ids and schema versions where consumers use them for
  compatibility.
- Change a policy input and its enforcing test or command together.
- Change a schema and its positive, negative, migration, or lockstep fixtures
  together.
- Keep release inputs aligned with the commands and package boundary they
  select.
- Keep suite files explicit about selected tests, advisory status, and slow or
  internal scope.
- Direct generated reports and command output to `artifacts/` unless a named
  producer owns a governed output under `docs/spec` or `docs/reports`.

Do not add an unconsumed JSON file as evidence that policy exists. If no code
or test reads the file, it is reference data at most and must not be described
as enforcement.

## Dependency Policy

`policy/dependency_rules.json` is a narrow forbidden-edge list consumed by the
maintainer dependency guard. It does not define the complete Cargo graph.
Workspace manifests, Cargo metadata, the foundation dependency-direction
contract, and focused boundary tests remain the complete dependency
authorities.

Rust source, license, and advisory policy lives under `configs/rust/` and
`audit-allowlist.toml`, not in this directory.

## Validation Anchors

- `crates/bijux-dev/src/commands/mod.rs`
- `crates/bijux-dev/src/commands/ops.rs`
- `crates/bijux-dev/src/commands/suite_dispatch.rs`
- `crates/bijux-dev/src/commands/release_validation_suite.rs`
- `crates/bijux-dev/tests/`
- `docs/bijux-dag/quality/dependency-governance.md`

Use focused consumer tests for a changed file. For cross-cutting DAG policy,
run the relevant `bijux-dev-dag` suite and retain its final status rather than
claiming validity from successful JSON parsing alone.
