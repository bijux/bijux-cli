# Integrations And Routed Runtimes

## Purpose

This page records the reference facts for product binary routing, plugin
runtime behavior, Python facade APIs, and the remaining Python compatibility
baseline that still matters.

```mermaid
flowchart TD
    A[bijux <tool>] --> B[bijux-<tool> runtime binary]
    C[bijux dev <tool>] --> D[bijux-dev-<tool> control binary]
    E[Python package] --> F[Rust-backed facade]
    G[plugins command group] --> H[plugin.manifest.json lifecycle]
```

```mermaid
flowchart LR
    A[Current Rust runtime] --> B[current bijux-cli-python]
    B --> C[configured PyPI baseline 0.2.0]
```

## Product Binary Routing

The `bijux` umbrella binary owns routed product execution.

| Surface | Binary pattern |
| --- | --- |
| Runtime route | `bijux <tool> ...` -> `bijux-<tool>` |
| Control-plane route | `bijux dev <tool> ...` -> `bijux-dev-<tool>` |

Known routed product namespaces are:

`agent`, `atlas`, `dag`, `dna`, `gnss`, `rag`, `rar`, `vex`

Configured product binary directories are checked before or after `PATH`
according to `BIJUXCLI_PRODUCT_BIN_PRECEDENCE`.

## Plugin Runtime Reference

The durable local plugin contract is based on `plugin.manifest.json`.

- local installs consume `plugin.manifest.json`
- `plugins info` reports registry-wide status and inventory details
- delegated and Python plugins resolve their declared entrypoint from the
  installed manifest anchor when available
- `plugins inspect [plugin]` can report either one plugin or the full
  inventory, and `plugins doctor` reports current-runtime drift and missing
  entrypoints
- `plugins explain [plugin]` can report either a registry-wide summary or one
  plugin's compatibility and load diagnostics
- `plugins where` reports the active plugins directory and registry file
- `plugins reserved-names` reports the current reserved plugin namespace
  inventory
- compatibility is validated from `compatibility.min_inclusive` and
  `compatibility.max_exclusive`
- duplicate namespaces and alias conflicts are rejected during install
- installed plugin namespaces are not currently executed as routed
  `bijux <plugin-namespace> ...` subcommands

## Python Facade APIs

The current documented Python-facing facade exports:

- `version()`
- `command_tree_introspection()`
- `execution_facade(argv)`
- `execution_facade_with_status(argv)`
- `output_envelope_model()`
- `error_to_exception(payload)`
- `config_resolution_helpers(home_dir)`
- `plugin_registry_inspection(registry_file)`
- `install_path_helpers(home_dir)`
- `migration_warnings()`
- `post_install_diagnostics()`

The Python package is a Rust-backed compatibility surface, not a second
independent runtime.

Relevant runtime-facing exceptions include:

- `PlatformWheelUnavailable`
- `NativeExtensionUnavailable`

## Local Product Routing

Adjacent Bijux product binaries can be routed through the umbrella command when
their binaries are discoverable.

Typical local setup:

```bash
export BIJUXCLI_PRODUCT_BIN_DIR="/path/to/product/bin"
```

Examples:

```bash
bijux atlas --help
bijux dev atlas --help
bijux dev cli list-products --format json --no-pretty
```

The `list-products` output is the verification surface for resolved runtime and
control-plane product binaries.

## Current Python Compatibility Baseline

Compatibility review is still anchored on two comparisons:

1. current `bijux-cli` vs current `bijux-cli-python`
2. current `bijux-cli-python` vs the repository's configured stable PyPI
   baseline, currently `bijux-cli==0.2.0`

Historically retained Python-facing overlap still matters for:

- top-level command identity
- global flag behavior and precedence
- plugin command behavior
- REPL and completion expectations
- documented exit code meanings

## Migration Notes That Still Matter

- the Python package now delegates command execution to the Rust runtime
- runtime resolution follows `BIJUX_BIN` first, then `bijux` on `PATH`
- compatibility warnings and post-install diagnostics exist to surface legacy
  Python-only assumptions
- if no usable runtime binary is found, the facade fails rather than silently
  inventing a second execution path

## Honest Limit

This page describes supported routed and compatibility surfaces. It does not
claim that every historical Python implementation detail remains public or
stable.
