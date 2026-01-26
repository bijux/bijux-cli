# Plugin Examples

## Purpose
These examples demonstrate how plugin operations behave in practice. They are designed to show how installation, inspection, and removal interact with the registry and filesystem.

## Scope
The examples assume a local plugin directory with minimal metadata. They do not cover plugin authoring or complex dependency resolution.

## Example 1: Install a Local Plugin
This example shows a basic local install workflow.

### Setup
Create a minimal plugin directory with metadata and a module file.

```bash
mkdir -p ./my_plugin
cat > ./my_plugin/plugin.json <<'JSON'
{
  "name": "my_plugin",
  "version": "1.0.0",
  "bijux_cli_version": ">=0",
  "schema_version": "1"
}
JSON
printf '# plugin\n' > ./my_plugin/plugin.py
```

### Command
```bash
bijux plugin install ./my_plugin
```

### Expected Output
The command reports a successful install and exits with code 0. The plugin is added to the registry and becomes available to the CLI.

### Why This Matters
Install validates metadata before activation, which prevents partial or broken plugins from entering the registry.

## Example 2: Inspect Installed Plugins
This example verifies that the registry reflects the installed plugin.

### Command
```bash
bijux plugin list
bijux plugin info my_plugin
```

### Expected Output
`list` includes `my_plugin`, and `info` prints metadata about the plugin. If the plugin is not present, the command fails with a stable error.

### Why This Matters
The registry is the single source of truth for installed plugins. Listing and inspecting confirm consistency with filesystem state.

## Example 3: Uninstall and Verify Cleanup
This example shows removal and validation that the registry is clean.

### Command
```bash
bijux plugin uninstall my_plugin
bijux plugin list
```

### Expected Output
Uninstall succeeds and `list` no longer shows the plugin. If removal fails, the CLI emits a structured error and preserves registry integrity.

### Why This Matters
Uninstall must be symmetric with install. Consistent cleanup prevents registry drift and avoids manual cleanup work.
