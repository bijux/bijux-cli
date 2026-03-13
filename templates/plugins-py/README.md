# Python Plugin Template

Render this template with Cookiecutter from the repository root:

```bash
python3 -m cookiecutter ./templates/plugins-py project_name="My Plugin" plugin_namespace=my-plugin
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
- `plugin.py`: delegated Python entrypoint exposed as `plugin:main`.
- `pyproject.toml`: optional local packaging metadata for the rendered project.

Keep the plugin namespace stable after release, update the compatibility range when supported host
versions change, and add tests before sharing the plugin. The default template values target
plugin version `0.3.0` with host compatibility from `0.3.0` up to, but not including, `1.0.0`.
Use `plugin_namespace` for the durable CLI name even when `project_name` is a human-readable title.
Cookiecutter validation rejects namespaces that do not start with a letter or that contain repeated
hyphens.
