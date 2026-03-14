# Plugin Templates

Repository-maintained Cookiecutter templates live under this directory for plugin authors who want
more than the built-in `bijux plugins scaffold` minimal layout.

- `plugins-py`: Python plugin template with the current `plugin.manifest.json` contract.
- `plugins-rs`: Cargo-backed Rust plugin template with an executable manifest entrypoint.

## Usage

```bash
python3 -m cookiecutter ./templates/plugins-py \
  project_name="My Plugin" \
  project_slug=my-plugin \
  plugin_namespace=my-plugin
bijux plugins install ./my-plugin
bijux plugins list
bijux plugins inspect my-plugin
bijux plugins check my-plugin
bijux plugins explain my-plugin
bijux plugins schema
bijux my-plugin --help
```

```bash
python3 -m cookiecutter ./templates/plugins-rs \
  project_name="My Plugin" \
  project_slug=my-plugin \
  plugin_namespace=my-plugin \
  crate_name=my_plugin
bijux plugins install ./my-plugin
bijux plugins list
bijux plugins inspect my-plugin
bijux plugins check my-plugin
bijux plugins explain my-plugin
bijux plugins schema
bijux my-plugin --help
```

These templates are rendered with Cookiecutter. Use them when you want a repository-shaped plugin
project with authoring files such as `README.md`, packaging metadata, and ignore rules. The built-in
`bijux plugins scaffold` command stays intentionally minimal and does not load custom templates.

The Rust template keeps local development honest: `plugin-entrypoint` rebuilds the debug binary on
first use or after source drift, then runs the compiled plugin. Python plugins require Python 3.11
or newer on `PATH`.

Keep the rendered plugin namespace stable after release, update the compatibility window when host
support changes, and avoid namespaces reserved by `bijux-cli` or official Bijux tools.
When `project_name` contains leading digits or presentation-only punctuation, pass explicit stable
`project_slug`, `plugin_namespace`, and `crate_name` values instead of relying on derived defaults.
New plugins still start at plugin version `0.1.0` because plugin semver belongs to the plugin's own
release history, not to the Bijux host release line. The template `cli_min` / `cli_max` values are
the host compatibility contract instead, and the current repository defaults target `0.3.0` up to,
but not including, `0.4.0`.
