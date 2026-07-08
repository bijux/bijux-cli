---
title: Badge Catalog
audience: maintainer
type: reference
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-07-09
---

# Badge Catalog

This page defines the shared badge language for the repository README, the docs
home page, and the public package surfaces.

Edit the templates here, then run `make sync-badges` so the root README, the
docs landing page, and every public package README stay aligned.

Do not hand-edit generated badge sections inside README or docs package pages.
Those surfaces consume the templates below through generated badge blocks.

The repository summary badges describe the public release surface package by
package. Package surfaces can still render package-specific badge sets for
narrower boundaries such as the Python bridge.

## Badge Order

Generated badge sections render in this order:

1. surface summary badges
2. one line of release-channel badges in this order: `crates.io`, `PyPI`, `GHCR`
3. one line of documentation badges in this order: repository docs, package docs, Rust docs

## Link Policy

GHCR badge links are fixed here as part of the contract:

- the repository-wide GHCR summary badge links to
  `https://github.com/bijux?tab=packages&repo_name=bijux-core`
- per-package GHCR badges render only for release bundles that publish
  containers today: `bijux-cli` and `bijux-dag-cli`
- public library crates still render crates.io, docs, and docs.rs badges even
  when they do not publish GHCR images

## Repository Summary

<!-- bijux-core-badges:repository-summary:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-core/workflows/deploy-docs/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-core/workflows/release-crates/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml)
[![PyPI Publish](https://github.com/bijux/bijux-core/workflows/release-pypi/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-core?display_name=tag&label=release)](https://github.com/bijux/bijux-core/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-{{ ghcr_package_count }}%20packages-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-core)
[![Published packages](https://img.shields.io/badge/published%20packages-{{ public_package_count }}-2563EB)](https://github.com/bijux/bijux-core/tree/main/crates)
<!-- bijux-core-badges:repository-summary:end -->

## Rust Package Summary

<!-- bijux-core-badges:rust-package-summary:start -->
[![Crates.io](https://img.shields.io/crates/v/{{ crate_name }}?label=crates.io&logo=rust)]({{ crates_url }})
[![Rust docs](https://img.shields.io/badge/rust--docs-{{ crate_badge_label }}-DEA584?logo=rust&logoColor=white)]({{ docsrs_url }})
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)
<!-- bijux-core-badges:rust-package-summary:end -->

## Python Package Summary

<!-- bijux-core-badges:python-package-summary:start -->
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-3776AB?logo=python&logoColor=white)]({{ pypi_url }})
[![PyPI](https://img.shields.io/pypi/v/{{ pypi_name }}?label=PyPI&logo=pypi)]({{ pypi_url }})
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/workflows/repo%20/%20ci/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)
<!-- bijux-core-badges:python-package-summary:end -->

## Family Crates Badge

<!-- bijux-core-badges:family-crates-badge:start -->
[![{{ badge_title }}](https://img.shields.io/crates/v/{{ crate_name }}?label={{ crate_badge_label }}&logo=rust)]({{ crates_url }})
<!-- bijux-core-badges:family-crates-badge:end -->

## Family Rustdocs Badge

<!-- bijux-core-badges:family-rustdocs-badge:start -->
[![{{ docsrs_badge_alt }}](https://img.shields.io/badge/rust--docs-{{ crate_badge_label }}-DEA584?logo=rust&logoColor=white)]({{ docsrs_url }})
<!-- bijux-core-badges:family-rustdocs-badge:end -->

## Family PyPI Badge

<!-- bijux-core-badges:family-pypi-badge:start -->
[![{{ badge_title }}](https://img.shields.io/pypi/v/{{ pypi_name }}?label={{ pypi_badge_label }}&logo=pypi)]({{ pypi_url }})
<!-- bijux-core-badges:family-pypi-badge:end -->

## Family GHCR Badge

<!-- bijux-core-badges:family-ghcr-badge:start -->
[![{{ badge_title }}](https://img.shields.io/badge/{{ ghcr_badge_label }}-ghcr-181717?logo=github)]({{ ghcr_url }})
<!-- bijux-core-badges:family-ghcr-badge:end -->

## Family Docs Badge

<!-- bijux-core-badges:family-docs-badge:start -->
[![{{ docs_badge_alt }}](https://img.shields.io/badge/docs-{{ docs_badge_label }}-2563EB?logo=materialformkdocs&logoColor=white)]({{ docs_url }})
<!-- bijux-core-badges:family-docs-badge:end -->

## Repository Docs Badge

<!-- bijux-core-badges:repository-docs-badge:start -->
[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/bijux-core/)
<!-- bijux-core-badges:repository-docs-badge:end -->
