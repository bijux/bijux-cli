# Plugin Write-Path Baseline

This document freezes the first Rust plugin write-path baseline.

## Baseline operations

- `install_plugin`
- `uninstall_plugin`
- `enable_plugin`
- `disable_plugin`

## Baseline guarantees

1. Registry writes are atomic.
2. Failed mutations do not partially corrupt existing registry state.
3. Reserved and future official namespaces are rejected on install.
4. Incompatible manifests are rejected on install.
5. Enabling plugins in `Broken` lifecycle state is rejected.
6. Missing-plugin operations return deterministic not-found errors.
7. List and inspect behavior remains deterministic after write failures.

## Version-change policy

- In-place upgrade/downgrade for same namespace is not part of this baseline.
- Version change requires explicit uninstall then install.

## Compatibility scope

- Python parity is guaranteed only where capture and command overlap is available.
- Unsupported parity surfaces must be documented before extension.
