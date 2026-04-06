# {{cookiecutter.project_name}}

Cargo-backed Rust plugin scaffold for Bijux.

- `plugin.manifest.json` defines the install contract.
- `plugin-entrypoint` is the executable manifest entrypoint. It rebuilds the local debug binary
  when Cargo inputs changed, refreshes `Cargo.lock` when needed, and then runs the compiled plugin
  through a locked dependency graph.
- `src/lib.rs` contains the plugin logic.
- `src/main.rs` exposes the compiled binary surface that `plugin-entrypoint` runs.
- `.gitignore` ignores the local Cargo build directory.

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

Run the generated plugin directly with:

```bash
./plugin-entrypoint --help
cargo build
cargo run -- --help
```

Keep `plugin-entrypoint` aligned with the compiled `{{cookiecutter.plugin_namespace}}` binary, keep
`plugin_namespace` stable after release, update the compatibility range in `plugin.manifest.json`
when supported Bijux host versions change, and avoid reserved Bijux namespaces when renaming the
plugin. The rendered defaults start at plugin version `{{cookiecutter.plugin_version}}` with host
compatibility from `{{cookiecutter.cli_min}}` up to, but not including,
`{{cookiecutter.cli_max}}`. For a pre-1.0 Bijux host, that upper bound should move with the next
supported minor line instead of every future `0.x` release. Plugin semver is separate from the
Bijux host release line, so the default plugin version remains
`{{cookiecutter.plugin_version}}` even when the host compatibility window advances. The Rust
library identifier remains `{{cookiecutter.crate_name}}`. The first local build creates
`Cargo.lock`, and later builds use `cargo build --locked`.
