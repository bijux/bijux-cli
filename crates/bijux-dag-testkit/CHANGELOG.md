# Changelog

All notable changes to **bijux-dag-testkit** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## 0.4.0 – 2026-07-04

### Added
- Declared `bijux-dag-testkit` as the shared repository-internal crate for deterministic DAG fixtures, fake adapters, and retained-run assertions.
- Reusable workflow builders, fixtures, and assertion helpers for DAG contract, integration, and regression suites.
- Shared support for cross-crate scenario coverage so graph, runtime, and application tests exercise the same canonical workflows.
