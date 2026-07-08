---
title: bijux-cli Package
audience: mixed
type: package
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# bijux-cli

<!-- bijux-core-badges:generated:start -->
[![Crates.io](https://img.shields.io/crates/v/bijux-cli?label=crates.io&logo=rust)](https://crates.io/crates/bijux-cli)
[![Rust docs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-dag](https://img.shields.io/crates/v/bijux-dag-cli?label=bijux--dag&logo=rust)](https://crates.io/crates/bijux-dag-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli) [![bijux-dag](https://img.shields.io/badge/bijux--dag-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-dag)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli) [![bijux-dag docs.rs](https://img.shields.io/badge/rust--docs-bijux--dag-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-dag-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-cli` is the public Rust runtime behind the `bijux` executable. It is
the source of truth for command semantics shared by the native binary, the
Python launcher, and the in-process SDK surfaces used by mounted apps and
tests.

Use this page when the question is about what the `bijux` runtime actually
does: parsing commands, normalizing inputs, executing features, shaping
envelopes, and preserving deterministic behavior across entrypoints.

## Reach For This Crate When

- the installed binary parses or renders something incorrectly
- a plugin, REPL flow, history path, config command, or runtime query behaves
  differently than expected
- a mounted app should use the same command semantics as the native runtime
- you need the stable Rust dependency that powers the visible `bijux` product

## What It Owns

| Surface | Ownership |
| --- | --- |
| command routing | parser, registry, command catalog, and normalization |
| runtime behavior | config, history, memory, install diagnostics, plugins, and REPL state |
| output contract | deterministic help, envelopes, and stream formatting |
| boundary | does not own maintainer control-plane commands or DAG semantics |

## What It Does Not Own

- Python packaging, interpreter discovery, and console-script distribution
  rules belong to [`bijux-cli-python`](bijux-cli-python.md).
- DAG authoring and graph execution semantics belong to the `bijux-dag-*`
  crates.
- Repository governance, release proof, and maintainer diagnostics belong to
  the maintainer tooling surfaces.

## Source Layout

- `crates/bijux-cli/src/api`
- `crates/bijux-cli/src/bootstrap`
- `crates/bijux-cli/src/contracts`
- `crates/bijux-cli/src/features`
- `crates/bijux-cli/src/infrastructure`
- `crates/bijux-cli/src/interface`
- `crates/bijux-cli/src/kernel`
- `crates/bijux-cli/src/routing`
- `crates/bijux-cli/src/shared`

## Practical Starting Points

- Open the [CLI Handbook](../index.md) when you need the product story before
  choosing a module.
- Open [CLI Interfaces](../interfaces/index.md) when the question is what
  callers can rely on.
- Open [CLI Operations](../operations/index.md) when the question is
  installation, diagnostics, release handling, or day-to-day runtime support.
- Open [`bijux-cli-python`](bijux-cli-python.md) when the issue is Python
  packaging, bridge parity, or launcher distribution.

## Code Anchors

- `crates/bijux-cli/README.md`
- `crates/bijux-cli/CHANGELOG.md`
- `crates/bijux-cli/src/bin`
- `crates/bijux-cli/tests/integration.rs`
- `crates/bijux-cli/tests/routing.rs`

## Review Focus

- runtime semantics should stay deterministic across binary and bridge entrypoints
- public command behavior should be explained in the CLI handbook, not hidden in tests
- maintainer-only concerns should not leak into this package boundary
