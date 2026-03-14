# Python Plugin Template

Render this template with Cookiecutter from the repository root:

```bash
python3 -m cookiecutter ./templates/plugins-py \
  project_name="My Plugin" \
  project_slug=my-plugin \
  plugin_namespace=my-plugin
```

The rendered project installs with the current plugin manifest contract:

```bash
bijux plugins install ./my-plugin
bijux plugins list
bijux plugins inspect my-plugin
bijux plugins check my-plugin
bijux plugins explain my-plugin
bijux plugins schema
bijux my-plugin --help
```

Generated files:

- `plugin.manifest.json`: current plugin contract consumed by install and diagnostics commands.
- `plugin.py`: delegated Python entrypoint exposed as `plugin:main`.
- `pyproject.toml`: optional local packaging metadata for the rendered project.
- `.gitignore`: ignores local Python cache and virtual environment state.

Keep the plugin namespace stable after release, update the compatibility range when supported host
versions change, and add tests before sharing the plugin. The default template values start new
plugins at version `0.1.0` and use the current `cli_min` / `cli_max` compatibility window from the
template defaults. For a pre-1.0 Bijux host, that window should stop at the next supported minor
line instead of claiming compatibility with every future `0.x` release. The plugin's own version is
intentionally independent from the Bijux host line, so it stays at `0.1.0` for a newly scaffolded
plugin even while the host compatibility window now targets `0.3.0` up to, but not including,
`0.4.0`.
Use `plugin_namespace` for the durable CLI name even when `project_name` is a human-readable title.
If `project_name` includes leading digits or punctuation that should not survive into identifiers,
pass `project_slug` and `plugin_namespace` explicitly.
Cookiecutter validation rejects namespaces that do not start with a letter or that contain repeated
hyphens, rejects invalid semver or inverted compatibility windows, and blocks namespaces reserved by
`bijux-cli` or official Bijux tools.
Python-backed plugin execution requires Python 3.11 or newer on `PATH`.
