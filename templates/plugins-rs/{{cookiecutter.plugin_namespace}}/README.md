# {{cookiecutter.project_name}}

Rust-backed delegated plugin scaffold for Bijux.

- `plugin.manifest.json` defines the install contract.
- `plugin.py` is the delegated host bridge entrypoint.
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
change, and avoid reserved Bijux namespaces when renaming the plugin.
