# {{cookiecutter.project_name}}

Python delegated plugin scaffold for Bijux.

- `plugin.manifest.json` defines the install contract.
- `plugin.py` exposes the `plugin:main` entrypoint.
- `pyproject.toml` keeps local packaging metadata with the project.

Install locally with:

```bash
bijux plugins install ./plugin.manifest.json
bijux plugins list
bijux plugins inspect {{cookiecutter.plugin_namespace}}
bijux plugins check {{cookiecutter.plugin_namespace}}
bijux plugins explain {{cookiecutter.plugin_namespace}}
bijux plugins schema
```

Keep `plugin_namespace` stable after release, update the compatibility range in
`plugin.manifest.json` when the supported Bijux host versions change, and avoid reserved Bijux
namespaces when renaming the plugin.
