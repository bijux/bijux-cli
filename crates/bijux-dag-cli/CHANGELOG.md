# Changelog

All notable changes to **bijux-dag-cli** are documented here.
This project adheres to [Semantic Versioning](https://semver.org) and the
[Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format.

## 0.4.0 – 2026-07-04

### Added
- First public crates.io release of `bijux-dag-cli`, which installs the `bijux-dag` executable.
- Stable local operator command surface for DAG validation, planning, execution, replay, artifact inspection, cache explanation, and verification.
- Thin binary wiring over `bijux-dag-app` so the installed command keeps a narrow, explicit ownership boundary.
