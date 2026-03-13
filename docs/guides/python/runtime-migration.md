# Python Runtime Migration

## Goal

Use the Rust-backed Python package runtime safely when migrating from legacy Python-only assumptions.

## What changed

- Python command execution now delegates to the Rust runtime path.
- Legacy Python-only behavior is no longer the execution source of truth.
- Compatibility warnings are exposed through `migration_warnings()` and post-install diagnostics.
- The Python package keeps a thin facade layer, but command execution, state
  resolution, and most contract enforcement happen in the Rust runtime.

## Runtime Resolution

The Python facade resolves the runtime in this order:

1. `BIJUX_BIN`
2. `bijux` in `PATH`

If no runtime binary is found, the facade raises
`PlatformWheelUnavailable`.

## Recommended migration checks

1. Verify script resolution:

```bash
which bijux
python -m bijux_cli_py version
bijux version
```

2. Verify diagnostics:

```bash
python - <<'PY'
from bijux_cli_py import post_install_diagnostics
print(post_install_diagnostics())
PY
```

3. Verify no shadowed duplicate install paths:

```bash
bijux cli doctor
```

## Failure handling expectations

- Missing runtime binary path: `PlatformWheelUnavailable`
- Failed extension import: `NativeExtensionUnavailable`

## Internal Differences That Still Matter

User-facing command behavior is meant to stay compatible, but the internals are
not the same as the old Python-only runtime:

- execution is Rust-backed
- the Python package is a compatibility facade
- plugin loading may fall back to subprocess delegation when direct loading is
  unavailable
- compatibility aliases may still emit deprecation or migration warnings

## Compatibility policy

Keep runtime invocation parity with Rust binary behavior for covered baseline commands. If behavior diverges, treat it as a regression and update parity tests before release.
