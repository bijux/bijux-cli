# Python Distribution Policy

## Binding Runtime

- Primary Python binding path is `PyO3 + maturin`.
- Wheels ship a Rust-backed extension module and a Python wrapper package.
- Python wrappers must fail over to a subprocess adapter when native extension loading is unavailable.

## Package Naming

- `bijux-cli` remains the canonical Python distribution for CLI runtime behavior.
- `bijux` is reserved for a compatibility/meta distribution that forwards users to the same runtime contract.
- Both names must resolve to the same user-facing `bijux` command semantics.

## Compatibility Promise

- Existing `pip install bijux-cli` users retain command-line compatibility.
- `bijux` executable ownership remains singular; wrappers must delegate to the same Rust command engine.
- Python API changes require additive compatibility shims or explicit deprecation messaging.
