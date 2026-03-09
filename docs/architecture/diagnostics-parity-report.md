# Diagnostics Parity Report

Date: 2026-03-09
Scope: tasks 301-320

## Implemented

### Inspect

- `inspect` and `cli inspect` now return enriched diagnostics payloads with:
  - route source metadata
  - alias rewrite metadata
  - namespace and built-in route introspection
  - plugin compatibility warnings and origin metadata
  - schema/version metadata section

### Dev diagnostics commands

- `dev cli routes` exposes source-of-route metadata and alias rewrite metadata.
- `dev cli registry` exposes namespace ownership grouping and precedence ordering.
- `dev cli env` exposes active compatibility paths and source precedence metadata.
- `dev cli doctor` reports normalized issue groups (`config`, `paths`, `plugins`) with stable shape.
- `dev cli contracts` exposes contract schema identifiers and runtime schema/version metadata.

### Test coverage added

- Inspect output mode coverage: text/json/yaml.
- Inspect trace and quiet mode behavior checks.
- Inspect failure normalization check for usage/help error path.
- Internal consistency check: `inspect.route_sources` equals `dev cli routes.routes`.
- Text snapshots for all dev diagnostics commands.
- JSON golden snapshots for all dev diagnostics commands (updated under `tests/snapshots/ported`).
- Core-level invocation assertions for diagnostics payload metadata.

## Status for 301-320

- `301`: complete (role audit documented)
- `302`: complete (inspect is fully implemented in Rust core)
- `303`: complete (inspect machine payload contract types defined)
- `304-306`: complete (inspect text/json/yaml tests + snapshots)
- `307`: complete (inspect trace-mode test)
- `308`: complete (inspect quiet-mode test)
- `309`: complete (inspect failure normalization test)
- `310`: complete (`dev cli routes` source metadata)
- `311`: complete (`dev cli registry` ownership metadata)
- `312`: complete (`dev cli env` precedence metadata)
- `313`: complete (`dev cli doctor` normalized issue groups)
- `314`: complete (`dev cli contracts` schema/version metadata)
- `315`: complete (golden snapshots for dev diagnostics commands)
- `316`: complete (JSON snapshots for dev diagnostics commands)
- `317`: complete (overlapping diagnostics parity checks)
- `318`: complete (inspect vs routes internal consistency check)
- `319`: complete (this report)
- `320`: complete (baseline frozen in `docs/architecture/diagnostics-baseline.md`)

## Remaining diagnostic gaps

- Python-captured inspect payload parity is limited by missing Python inspect golden captures.
- Additional plugin runtime diagnostics detail can be added after baseline lock if schema-compatible.
