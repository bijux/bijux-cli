# Contributor Engineering Rules

## Purpose
This document defines contributor and reviewer rules that keep milestone claims tied to evidence.

## Rules
1. Do not use completion language without parity, runtime, and test artifacts.
2. Reject hype wording that does not cite concrete generated evidence.
3. Route maintainer automation through `bijux dev cli` commands by default.
4. Treat behavior contracts as stable; treat file and doc counts as non-contracts.
5. Keep docs focused on law and change; move volatile status detail to generated artifacts.

## Required Evidence
- `artifacts/parity/command_parity_matrix.json`
- `artifacts/parity/rust_python_parity_report.json`
- `artifacts/status/current_rust_state.json`
- `artifacts/status/docs_audit.json`
- `artifacts/status/test_quality_audit.json`

## Review Gate
A milestone statement is rejectable if it does not include done/left/blocked evidence.
