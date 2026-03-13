# Plugins

Plugins are installed from a manifest, registered in the plugin registry, and
resolved by namespace at runtime.

## Common Commands

```bash
bijux cli plugins list
bijux cli plugins inspect NAMESPACE
bijux cli plugins install ./plugin.manifest.json
bijux cli plugins check NAMESPACE
```

## References

- [Plugin examples](../examples/plugins.md)
- [Plugin lifecycle concept](../concepts/plugin-lifecycle.md)
- [Plugin state](../plugin_state.md)
- [Plugin write-path parity report](../architecture/plugin-write-path-parity-report.md)

## Notes

- Keep the manifest file as the installation source of truth.
- Use `inspect` and `check` before assuming a plugin is healthy.
- Use the examples page for scaffold and install walkthroughs.
