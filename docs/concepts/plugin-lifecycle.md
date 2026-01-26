# Plugin lifecycle

Lifecycle states:

- discovered
- installed
- active
- inactive
- removed

Transition rules:

- discovered -> installed -> active
- active <-> inactive
- active/inactive -> removed

Rollback guarantees:

- Failed activation leaves registry and filesystem clean
- Invalid metadata fails before activation

Compatibility rules:

- Plugins must declare CLI compatibility
- Incompatible plugins are rejected

Plugins may NOT:

- mutate core policy resolution
- bypass output routing rules

## Contracts

Plugin contracts are enforced before activation.

- Registry: `RegistryProtocol`
- Plugin config: `PluginConfig`
- Plugin interface: `RegistryProtocol` command registration rules

## Stage contract

### Install

- Input: plugin directory or package name
- Output: structured payload with status and plugin list
- Failures: invalid name, install failure, missing entry point, incompatible CLI

### Verify

- Validates metadata (`plugin.json` or entry points)
- Rejects duplicates or incompatible versions

### List

- Output: structured list of installed plugins
- Failures: invalid format flag, inaccessible plugins dir

### Info

- Output: structured metadata for a single plugin
- Failures: missing plugin, invalid metadata, invalid format flag

### Check

- Output: healthy or unhealthy status
- Failures: missing plugin, invalid health hook, invalid format flag

### Load

- Imports plugin code and registers commands
- Failures: import or initialization errors

### Unload

- Removes plugin from active registry
- Failures: cleanup errors are logged only

### Uninstall

- Output: structured payload with uninstall status
- Failures: missing plugin, uninstall failure, filesystem errors
