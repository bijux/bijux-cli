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

The repository summary shows only repository-level health and release signals.
Package surfaces show only the package's own registry and documentation
signals. The generated package map carries cross-package discovery without
turning every page header into a release dashboard.

## Badge Order

Generated badge sections render in this order:

1. repository or package summary badges
2. repository and package handbook links on package surfaces

## Link Policy

The generated package map owns registry, container, docs.rs, handbook, and
source links for the complete public package family. Package headers do not
repeat links for sibling packages.

## Repository Summary

<!-- bijux-core-badges:repository-summary:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![Docs](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml?query=branch%3Amain)
[![Crates Publish](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml)
[![PyPI Publish](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml)
[![GHCR Publish](https://github.com/bijux/bijux-core/actions/workflows/release-ghcr.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/release-ghcr.yml)
[![GitHub Release](https://github.com/bijux/bijux-core/actions/workflows/release-github.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/release-github.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-core?display_name=tag&label=release)](https://github.com/bijux/bijux-core/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-{{ ghcr_package_count }}%20packages-181717?logo=github)](https://github.com/bijux?tab=packages&repo_name=bijux-core)
[![Public crates](https://img.shields.io/badge/public%20crates-{{ public_crate_count }}-2563EB)](https://github.com/bijux/bijux-core/tree/main/crates)
<!-- bijux-core-badges:repository-summary:end -->

## Rust Package Summary

<!-- bijux-core-badges:rust-package-summary:start -->
[![Crates.io](https://img.shields.io/crates/v/{{ crate_name }}?label=crates.io&logo=rust)]({{ crates_url }})
[![Rust docs](https://img.shields.io/badge/rust--docs-{{ crate_badge_label }}-DEA584?logo=rust&logoColor=white)]({{ docsrs_url }})
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)
<!-- bijux-core-badges:rust-package-summary:end -->

## Python Package Summary

<!-- bijux-core-badges:python-package-summary:start -->
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-3776AB?logo=python&logoColor=white)]({{ pypi_url }})
[![PyPI](https://img.shields.io/pypi/v/{{ pypi_name }}?label=PyPI&logo=pypi)]({{ pypi_url }})
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml?query=branch%3Amain)
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
[![Repository docs](https://img.shields.io/badge/docs-repository-2563EB?logo=materialformkdocs&logoColor=white)](https://bijux.io/bijux-core/)
<!-- bijux-core-badges:repository-docs-badge:end -->
