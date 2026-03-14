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
bijux plugins install ./my-plugin/plugin.manifest.json
bijux plugins list
bijux plugins inspect my-plugin
bijux plugins check my-plugin
bijux plugins explain my-plugin
bijux plugins schema
```

```bash
python3 -m cookiecutter ./templates/plugins-rs \
  project_name="My Plugin" \
  project_slug=my-plugin \
  plugin_namespace=my-plugin \
  crate_name=my_plugin
bijux plugins install ./my-plugin/plugin.manifest.json
bijux plugins list
bijux plugins inspect my-plugin
bijux plugins check my-plugin
bijux plugins explain my-plugin
bijux plugins schema
```

These templates are rendered with Cookiecutter. The built-in `bijux plugins scaffold` command does
not load custom templates.

Keep the rendered plugin namespace stable after release, update the compatibility window when host
support changes, and avoid namespaces reserved by `bijux-cli` or official Bijux tools.
When `project_name` contains leading digits or presentation-only punctuation, pass explicit stable
`project_slug`, `plugin_namespace`, and `crate_name` values instead of relying on derived defaults.
