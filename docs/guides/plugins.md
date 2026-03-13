# Plugins

Plugins are installed from a manifest, registered in the plugin registry, and
resolved by namespace at runtime.

## Common Commands

```bash
bijux cli plugins list
bijux cli plugins inspect NAMESPACE
bijux cli plugins install ./plugin.manifest.json
bijux cli plugins check NAMESPACE
bijux cli plugins schema
```

## References

- [Plugin examples](../examples/plugins.md)
- [Concepts overview](../concepts/index.md)
- [Plugin state](../plugin_state.md)
- [Plugin write-path parity report](../architecture/plugin-write-path-parity-report.md)

## Notes

- Keep the manifest file as the installation source of truth.
- Use `inspect` and `check` before assuming a plugin is healthy.
- Use the examples page for scaffold and install walkthroughs.

## Python And Delegated Plugins

For local delegated execution:

- declare plugin kind as `python` or `delegated`
- keep the manifest compatibility window current
- use a stable `module:function` entrypoint

Compatibility and trust metadata appear in plugin inspection output. Local
installs should keep the manifest file available so the runtime can resolve the
entrypoint from the installed manifest anchor.
