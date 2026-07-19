---
title: bijux-dev Package
audience: maintainers
type: package
status: canonical
owner: bijux-dev-docs
last_reviewed: 2026-07-19
---

# bijux-dev

`bijux-dev` is the private maintainer package for this repository. It converts
repository policy, public product contracts, retained evidence, and release
requirements into inspectable commands and tests. It is not an end-user
product and is not published as an installable package.

## Binary Authorities

| Binary | Authoritative responsibilities | Typical result |
| --- | --- | --- |
| `bijux-dev-cli` | repository status, runtime and package diagnostics, documentation publishing, maintenance audits, and cross-surface parity views | text, JSON, or YAML observations |
| `bijux-dev-dag` | governed suite discovery, policy and contract execution, evidence verification, DAG diagnostics, and release-proof composition | validation envelopes, governed evidence, and aggregate process status |

These binaries are not aliases. `bijux-dev-cli` presents repository and
product observations. `bijux-dev-dag` composes enforceable governance. The
visible `bijux-dev-dag` root command surface is governed by
`contracts/foundation/maintainer_command_surface.v1.json`.

## Package Boundary

The package may depend on product crates and read their public contracts to
detect repository-wide drift. Product crates must not depend on `bijux-dev`.

`bijux-dev` owns:

- repository layout, dependency, documentation, and policy checks;
- suite discovery, selection, explanation, and aggregate status;
- generated governance and release evidence with explicit source identity;
- maintainer reports that inspect product facts without redefining them.

It does not own `bijux` routing or plugin behavior, DAG execution semantics,
Python bridge behavior, or organization-wide workflow policy synchronized
from `bijux-std`.

## Source Ownership

| Path | Responsibility |
| --- | --- |
| `crates/bijux-dev/src/maintainer/` | repository and product diagnostic report composition |
| `crates/bijux-dev/src/commands/` | `bijux-dev-dag` command behavior and governed command families |
| `crates/bijux-dev/src/suites/` | reusable suite definitions, metadata, and selection |
| `crates/bijux-dev/src/repo/` | repository inspection and repository-owned operations |
| `crates/bijux-dev/src/report/` | shared report and evidence presentation |
| `crates/bijux-dev/src/bin/bijux-dev-cli.rs` | diagnostic binary entrypoint |
| `crates/bijux-dev/src/main.rs` | governance binary entrypoint |
| `crates/bijux-dev/tests/` | architecture, command, policy, evidence, and release contracts |

Product behavior belongs in its product crate even when a maintainer contract
is the first test that detects the defect.

## Result Semantics

- Required validation exits non-zero when any selected required check fails.
- Advisory, slow, internal, and narrowed selections remain explicit in the
  result.
- Broad suites preserve component failures and produce an aggregate terminal
  status instead of turning partial execution into success.
- A process start, generated path, or individual report is not proof of
  success without final status and integrity review.
- Transient output belongs under `artifacts/`; checked-in generated output
  requires a named producer and a contract test.

## Verify The Boundary

```bash
cargo test -p bijux-dev --test foundation_maintainer_command_surface_contracts
cargo test -p bijux-dev --test docs_source_reference_contracts
```

These focused tests verify the package's documented surface and references.
They do not prove that the full Rust, Python, documentation, or release lanes
passed.

## Maintainer References

- [Package README](../../../crates/bijux-dev/README.md)
- [Package contract](../../../crates/bijux-dev/CONTRACT.md)
- [Command Surface](../operations/command-surface.md)
- [Repository Gates](../operations/repository-gates.md)
- [Evidence Collection](../operations/evidence-collection.md)
- [Maintainer Handbook](../index.md)
