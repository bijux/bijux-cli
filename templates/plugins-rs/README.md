# Rust Plugin Template

Render this template with Cookiecutter from the repository root:

```bash
python3 -m cookiecutter ./templates/plugins-rs \
  project_name="My Plugin" \
  project_slug=my-plugin \
  plugin_namespace=my-plugin \
  crate_name=my_plugin
```

The rendered project installs with the current plugin manifest contract:

```bash
bijux plugins install ./my-plugin/plugin.manifest.json
bijux plugins list
bijux plugins inspect my-plugin
bijux plugins check my-plugin
bijux plugins explain my-plugin
bijux plugins schema
```

Generated files:

- `plugin.manifest.json`: current plugin contract consumed by install and diagnostics commands.
- `plugin.py`: placeholder bridge stub referenced by `plugin.manifest.json`.
- `Cargo.toml` and `src/lib.rs`: Rust baseline for the durable implementation behind the bridge.

Keep `plugin.py` aligned with your Rust bridge, keep the manifest namespace stable after release,
and update the compatibility range when the supported Bijux host versions change. The default
template values start new plugins at version `0.1.0` and use the current `cli_min` / `cli_max`
compatibility window from the template defaults.
Use `plugin_namespace` for the durable CLI name and `crate_name` for the Cargo identifier when
`project_name` contains presentation-only punctuation.
If `project_name` includes leading digits or punctuation that should not survive into identifiers,
pass `project_slug`, `plugin_namespace`, and `crate_name` explicitly.
Cookiecutter validation rejects namespaces that do not start with a letter, crate names that are
not lowercase snake_case, invalid semver or inverted compatibility windows, and Rust keyword crate
names. It also blocks namespaces reserved by `bijux-cli` or official Bijux tools.
The generated `plugin.py` is intentionally only a placeholder bridge stub until you wire it to a
real Rust entrypoint.
