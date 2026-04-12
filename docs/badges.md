---
title: Badge Catalog
audience: maintainer
type: reference
status: canonical
owner: bijux-core-docs
last_reviewed: 2026-04-12
---

# Badge Catalog

`docs/badges.md` is the single source of truth for shared badge templates across
the repository README and documentation landing surfaces.

Update the named templates here, then run `make sync-badges` so the root README,
the docs landing page, and every public package README publish the same badge
contract.

Do not hand-edit generated badge sections inside README or docs package pages.
Those surfaces consume the templates below through generated badge blocks.

The root README package map tracks public release families, while package
README and docs surfaces can still render package-specific badge sets for
implementation boundaries such as the Python bridge.

Generated badge sections always render in this order:

1. surface summary badges
2. one line of `crates.io` badges for every public Rust package
3. one line of `docs.rs` badges for every public Rust package
4. one line of `PyPI` badges for every public Python package
5. one line of `GHCR` badges for every published release bundle
6. one line of documentation badges for every public package surface

Link policy for GHCR badges is fixed here as part of the contract:

- the repository-wide GHCR summary badge links to
  `https://github.com/bijux?tab=packages`
- per-package GHCR badges link to the package-specific
  `https://github.com/bijux/bijux-core/pkgs/container/bijux-core%2Fbijux-cli`
  page

## Repository Summary

<!-- bijux-core-badges:repository-summary:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml)
[![Docs](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/deploy-docs.yml)
[![Crates Publish](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-crates.yml)
[![PyPI Publish](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/release-pypi.yml)
[![Release](https://img.shields.io/github/v/release/bijux/bijux-core?display_name=tag&label=release)](https://github.com/bijux/bijux-core/releases)
[![GHCR packages](https://img.shields.io/badge/ghcr-{{ ghcr_package_count }}%20package-181717?logo=github)](https://github.com/bijux?tab=packages)
[![Published packages](https://img.shields.io/badge/published%20packages-{{ public_package_count }}-2563EB)](https://github.com/bijux/bijux-core/tree/main/crates)
<!-- bijux-core-badges:repository-summary:end -->

## Rust Package Summary

<!-- bijux-core-badges:rust-package-summary:start -->
[![Crates.io](https://img.shields.io/crates/v/{{ crate_name }}?label=crates.io&logo=rust)]({{ crates_url }})
[![Docs.rs](https://img.shields.io/docsrs/{{ crate_name }}?label=docs.rs)]({{ docsrs_url }})
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)
<!-- bijux-core-badges:rust-package-summary:end -->

## Python Package Summary

<!-- bijux-core-badges:python-package-summary:start -->
[![Python 3.11+](https://img.shields.io/badge/python-3.11%2B-3776AB?logo=python&logoColor=white)]({{ pypi_url }})
[![PyPI](https://img.shields.io/pypi/v/{{ pypi_name }}?label=PyPI&logo=pypi)]({{ pypi_url }})
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-core/blob/main/LICENSE)
[![CI Status](https://github.com/bijux/bijux-core/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-core/actions/workflows/ci.yml)
[![GitHub Repository](https://img.shields.io/badge/github-bijux%2Fbijux--core-181717?logo=github)](https://github.com/bijux/bijux-core)
<!-- bijux-core-badges:python-package-summary:end -->

## Family Crates Badge

<!-- bijux-core-badges:family-crates-badge:start -->
[![{{ badge_title }}](https://img.shields.io/crates/v/{{ crate_name }}?label={{ crate_badge_label }}&logo=rust)]({{ crates_url }})
<!-- bijux-core-badges:family-crates-badge:end -->

## Family Rustdocs Badge

<!-- bijux-core-badges:family-rustdocs-badge:start -->
[![{{ docsrs_badge_alt }}](https://img.shields.io/docsrs/{{ crate_name }}?label={{ docsrs_badge_label }})]({{ docsrs_url }})
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
