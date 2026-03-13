# Plugin Command and Lifecycle Behavior

This page records the current plugin command surface that was previously owned by the Python
runtime lineage. The durable local plugin contract is now based on `plugin.manifest.json` plus the
declared entrypoint module, not the older `plugin.json` metadata file.

## Current command inventory
- `plugins list`
- `plugins info`
- `plugins inspect`
- `plugins check`
- `plugins enable`
- `plugins disable`
- `plugins install`
- `plugins uninstall`
- `plugins scaffold`
- `plugins doctor`
- `plugins reserved-names`
- `plugins where`
- `plugins explain`
- `plugins schema`

## Current local plugin behavior
- Local installs consume `plugin.manifest.json`.
- Delegated and Python plugins resolve `plugin:main`-style entrypoints from the installed manifest
  directory when provenance is available.
- Compatibility is validated from `compatibility.min_inclusive` and `compatibility.max_exclusive`.
- Duplicate namespaces and alias conflicts are rejected during install.
