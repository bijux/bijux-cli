# Plugin Command and Lifecycle Behavior

## Source of truth
- `src/bijux_cli/cli/plugins/commands/__init__.py`
- `src/bijux_cli/cli/plugins/commands/*.py`
- `src/bijux_cli/plugins/metadata.py`
- `src/bijux_cli/plugins/__init__.py`
- `docs/concepts/plugin-lifecycle.md`

## Plugin command inventory
- `plugins scaffold`
- `plugins install`
- `plugins uninstall`
- `plugins list`
- `plugins info`
- `plugins check`

## Discovery and metadata behavior
- Discovers entry-point plugins from group `bijux_cli.plugins`.
- Discovers local plugins from plugins directory entries containing `plugin.py` and `plugin.json`.
- Validates metadata fields including name, schema version, and host compatibility requirement.
- Rejects duplicate plugin names across discovery sources.

## Lifecycle behavior inventory
- Install: supports local directory source and package install path.
- Uninstall: removes installed plugin (filesystem or pip uninstall for entry-point package origin).
- Check: imports plugin health hook and maps health result to status and exit code.
- List/info: surface plugin metadata and registration state.
- Cache invalidation occurs during lifecycle mutations to ensure fresh metadata resolution.
