# {{cookiecutter.project_name}}

Cargo-backed Rust plugin scaffold for Bijux.

- `plugin.manifest.json` defines the install contract.
- `plugin-entrypoint` is the executable entrypoint referenced by the manifest.
- `src/lib.rs` contains the plugin logic.
- `src/main.rs` exposes the binary surface that `plugin-entrypoint` runs.

Install locally with:

```bash
bijux plugins install ./plugin.manifest.json
bijux plugins list
bijux plugins inspect {{cookiecutter.plugin_namespace}}
bijux plugins check {{cookiecutter.plugin_namespace}}
bijux plugins explain {{cookiecutter.plugin_namespace}}
bijux plugins schema
```

Run the generated plugin directly with:

```bash
./plugin-entrypoint --help
cargo run -- --help
```

Keep `plugin-entrypoint` aligned with the Cargo binary, keep `plugin_namespace` stable after
release, update the compatibility range in `plugin.manifest.json` when supported Bijux host
versions change, and avoid reserved Bijux namespaces when renaming the plugin. The rendered
defaults start at plugin version `{{cookiecutter.plugin_version}}` with host compatibility from
`{{cookiecutter.cli_min}}` up to, but not including, `{{cookiecutter.cli_max}}`.
