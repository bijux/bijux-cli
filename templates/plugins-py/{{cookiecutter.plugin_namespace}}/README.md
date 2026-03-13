# {{cookiecutter.project_name}}

Python delegated plugin scaffold for Bijux.

- `plugin.manifest.json` defines the install contract.
- `plugin.py` exposes the `plugin:main` entrypoint.
- `pyproject.toml` keeps local packaging metadata with the project.

Install locally with:

```bash
bijux plugins install ./plugin.manifest.json
```
