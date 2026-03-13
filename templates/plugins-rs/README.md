# Rust Plugin Template

Render this template with Cookiecutter from the repository root:

```bash
python3 -m cookiecutter ./templates/plugins-rs project_name="My Plugin" plugin_namespace=my-plugin
```

The rendered project installs with the current plugin manifest contract:

```bash
bijux plugins install ./my-plugin/plugin.manifest.json
bijux plugins list
bijux plugins check my-plugin
bijux plugins explain my-plugin
```

Generated files:

- `plugin.manifest.json`: current plugin contract consumed by install and diagnostics commands.
- `plugin.py`: delegated entrypoint referenced by `plugin.manifest.json`.
- `Cargo.toml` and `src/lib.rs`: Rust baseline for the durable implementation behind the bridge.

Keep `plugin.py` aligned with your Rust bridge, keep the manifest namespace stable after release,
and update the compatibility range when the supported Bijux host versions change. The default
template values target plugin version `0.3.0` with host compatibility from `0.3.0` up to, but
not including, `1.0.0`.
Use `plugin_namespace` for the durable CLI name and `crate_name` for the Cargo identifier when
`project_name` contains presentation-only punctuation.
