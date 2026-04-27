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

[![bijux-cli](https://img.shields.io/crates/v/bijux-cli?label=bijux--cli&logo=rust)](https://crates.io/crates/bijux-cli) [![bijux-cli](https://img.shields.io/pypi/v/bijux-cli?label=bijux--cli&logo=pypi)](https://pypi.org/project/bijux-cli/) [![bijux-cli](https://img.shields.io/badge/bijux--cli-ghcr-181717?logo=github)](https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli)

[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/) [![bijux-cli docs](https://img.shields.io/badge/docs-bijux--cli-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-cli/packages/bijux-cli/) [![bijux-cli docs.rs](https://img.shields.io/badge/rust--docs-bijux--cli-DEA584?logo=rust&logoColor=white)](https://docs.rs/bijux-cli)
<!-- bijux-core-badges:generated:end -->

`bijux-cli` is the public Rust runtime behind the `bijux` executable. It owns
command parsing, normalization, registry lookup, execution flow, plugin-facing
runtime behavior, and the REPL surface.

Use this page when the question is about runtime behavior of the command
product itself rather than repository policy or Python distribution.

## Responsibility Map

| Surface | Ownership |
| --- | --- |
| command routing | parser, registry, command catalog, and normalization |
| runtime behavior | config, history, memory, install diagnostics, plugins, and REPL state |
| output contract | deterministic help, envelopes, and stream formatting |
| boundary | does not own maintainer control-plane commands or DAG semantics |

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

## Open Next

- open the [CLI Handbook](../../index.md) for architecture, interfaces,
  operations, and quality guidance
- open the [Repository Handbook](../../bijux-core/index.md) when a change
  crosses into DAG, maintainer, or repository governance concerns
- open [`bijux-cli-python`](./bijux-cli-python.md) when the question is
  Python packaging, bridge parity, or launcher distribution

## Code Anchors

- `crates/bijux-cli/README.md`
- `crates/bijux-cli/CHANGELOG.md`
- `crates/bijux-cli/src/bin`
- `crates/bijux-cli/tests/integration.rs`
- `crates/bijux-cli/tests/routing.rs`

## Review Lens

- runtime semantics should stay deterministic across binary and bridge entrypoints
- public command behavior should be explained in the CLI handbook, not hidden in tests
- maintainer-only concerns should not leak into this package boundary
