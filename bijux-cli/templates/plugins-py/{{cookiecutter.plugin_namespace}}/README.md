# {{cookiecutter.project_name}}

Python plugin scaffold for Bijux.

- `plugin.manifest.json` defines the install contract.
- `plugin.py` exposes the `plugin:main` entrypoint.
- `pyproject.toml` keeps local packaging metadata with the project.
- `.gitignore` ignores local Python cache and virtual environment state.

Install locally with:

```bash
bijux plugins install .
bijux plugins list
bijux plugins inspect {{cookiecutter.plugin_namespace}}
bijux plugins check {{cookiecutter.plugin_namespace}}
bijux plugins explain {{cookiecutter.plugin_namespace}}
bijux plugins schema
bijux {{cookiecutter.plugin_namespace}} --help
```

Keep `plugin_namespace` stable after release, update the compatibility range in
`plugin.manifest.json` when the supported Bijux host versions change, and avoid reserved Bijux
namespaces when renaming the plugin. The rendered defaults start at plugin version
`{{cookiecutter.plugin_version}}` with host compatibility from `{{cookiecutter.cli_min}}` up to,
but not including, `{{cookiecutter.cli_max}}`. For a pre-1.0 Bijux host, that upper bound should
move with the next supported minor line instead of every future `0.x` release. Plugin semver is
separate from the Bijux host release line, so the default plugin version remains
`{{cookiecutter.plugin_version}}` even when the host compatibility window advances.
Python-backed plugin execution requires Python 3.11 or newer on `PATH`.
