# bijux-dev

`bijux-dev` is the private maintainer package for the `bijux-core` repository.
It turns repository policy, product contracts, retained evidence, and release
requirements into inspectable commands and test suites.

The package is intentionally not published. End users should install
`bijux-cli` or the relevant `bijux-dag` package instead.

## Choose A Binary

| Binary | Owns | Typical output |
| --- | --- | --- |
| `bijux-dev-cli` | repository status, runtime/package diagnostics, documentation publishing, maintenance audits, and cross-surface parity views | text, JSON, or YAML maintainer reports |
| `bijux-dev-dag` | governed checks, tests, contracts, evidence verification, DAG diagnostics, release proof, and suite catalogs | validation envelopes, generated governed evidence, and process status |

The binaries are complementary, not aliases. `bijux-dev-cli` presents
repository and product observations. `bijux-dev-dag` composes enforceable
governance and evidence suites. A similarly named command in one binary is not
permission to duplicate the other's behavior.

Discover the live surfaces with:

```bash
cargo run -p bijux-dev --bin bijux-dev-cli -- --help
cargo run -p bijux-dev --bin bijux-dev-dag -- --help
```

The exact visible `bijux-dev-dag` root command order is governed by
[`contracts/foundation/maintainer_command_surface.v1.json`](../../contracts/foundation/maintainer_command_surface.v1.json).

## Package Boundary

This package may depend on product crates to inspect their public facts and
verify cross-surface contracts. Product crates must not depend on
`bijux-dev`.

`bijux-dev` owns:

- repository layout, dependency, documentation, and policy checks;
- suite discovery, selection, explanation, and aggregate status;
- generated governance and release evidence with explicit source identity;
- release-readiness and compatibility verification;
- maintainer diagnostics that read product contracts without redefining them.

It does not own:

- `bijux` command routing, plugin behavior, or user state;
- graph semantics, scheduling, backend execution, or artifact meaning;
- Python bridge runtime behavior;
- GitHub workflow policy synchronized from `bijux-std`.

## Validation And Mutation

Most commands validate or report and must not modify governed source. Commands
that generate checked-in references or reports are explicit about their output
paths and producer.

- Validation commands return non-zero when required selected checks fail.
- Advisory selection changes enforcement and must not be reported as a required
  gate pass.
- Generated local logs, run products, and one-off reports belong under
  `artifacts/`.
- Checked-in output under `docs/reports`, `docs/spec`, or another governed path
  must have an identifiable producer and contract test.
- A process start, report path, or generated file is not evidence of success
  without final status and integrity review.

## Source Ownership

| Path | Responsibility |
| --- | --- |
| `src/commands/` | `bijux-dev-dag` command behavior and governed command families |
| `src/suites/` | reusable suite definitions, metadata, and selection |
| `src/maintainer/` | repository and product diagnostic report composition |
| `src/repo/` | repository inspection and repository-owned operations |
| `src/report/` | shared report and evidence presentation |
| `src/bin/bijux-dev-cli.rs` | `bijux-dev-cli` process entrypoint |
| `src/main.rs` | `bijux-dev-dag` process entrypoint |
| `tests/` | architecture, command, policy, evidence, and release contracts |

Product behavior belongs in the product crate even when a maintainer test is
the first place that exposes drift.

## Verification

Run focused tests by owning test binary while editing. The root make targets
compose broader required lanes:

```bash
cargo test -p bijux-dev --test docs_source_reference_contracts
make lint
make test
```

Do not describe a focused `bijux-dev` test as proof that all runtime, Python,
documentation, or release lanes passed.

## Maintainer References

- [Package contracts](docs/CONTRACTS.md)
- [Package changelog](./CHANGELOG.md)
- [Maintainer handbook](../../docs/bijux-dev/index.md)
- [Command surface](../../docs/bijux-dev/operations/command-surface.md)
- [Repository gates](../../docs/bijux-dev/operations/repository-gates.md)
- [Evidence collection](../../docs/bijux-dev/operations/evidence-collection.md)
