# {{cookiecutter.project_name}}

Rust-backed plugin scaffold for Bijux.

- `plugin.manifest.json` defines the install contract.
- `plugin.py` is a placeholder bridge stub for the host entrypoint.
- `src/lib.rs` is the starting point for the Rust implementation.

Install locally with:

```bash
bijux plugins install ./plugin.manifest.json
bijux plugins list
bijux plugins inspect {{cookiecutter.plugin_namespace}}
bijux plugins check {{cookiecutter.plugin_namespace}}
bijux plugins explain {{cookiecutter.plugin_namespace}}
bijux plugins schema
```

Keep `plugin.py` aligned with the Rust bridge, keep `plugin_namespace` stable after release,
update the compatibility range in `plugin.manifest.json` when supported Bijux host versions
change, and avoid reserved Bijux namespaces when renaming the plugin. The rendered defaults start
at plugin version `{{cookiecutter.plugin_version}}` with host compatibility from
`{{cookiecutter.cli_min}}` up to, but not including, `{{cookiecutter.cli_max}}`. The generated
`plugin.py` is intentionally only a placeholder bridge stub until you wire it to a real Rust
entrypoint.
