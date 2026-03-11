# Python Baseline Parity Inputs

This directory is the single source of truth for Python-origin baseline inputs used by Rust parity tests.

Rules:
- Keep files deterministic and reviewable.
- Store only stable baseline inputs and expected payloads.
- Do not write runtime-generated reports here.

Current baseline payload snapshots remain under `tests/data/golden/ported/`.
As parity coverage is expanded, add canonical Python baseline inputs in this directory.
