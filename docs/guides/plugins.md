# Plugins

Goal: install and debug a plugin.

```bash
bijux plugin install ./my_plugin
bijux plugin list
bijux plugin info my_plugin
bijux plugin uninstall my_plugin
```

If install fails, check metadata and CLI compatibility.

Default install location is `~/.bijux/.plugins`. Override with `BIJUXCLI_PLUGINS_DIR`.
