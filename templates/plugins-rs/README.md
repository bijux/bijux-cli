# Rust Plugin Template

Render this template with Cookiecutter from the repository root:

```bash
python3 -m cookiecutter ./templates/plugins-rs \
  project_name="My Plugin" \
  project_slug=my-plugin \
  plugin_namespace=my-plugin \
  crate_name=my_plugin
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
- `plugin-entrypoint`: executable manifest entrypoint that builds the local debug binary when Cargo
  inputs changed and then runs the compiled plugin.
- `Cargo.toml`, `src/lib.rs`, and `src/main.rs`: Cargo-backed Rust implementation and CLI surface.
- `.gitignore`: ignores the local Cargo build directory.

Run the generated plugin locally with:

```bash
./plugin-entrypoint --help
cargo build
cargo run -- --help
```

Keep `plugin-entrypoint` aligned with the compiled binary name, keep the manifest namespace stable
after release, and update the compatibility range when the supported Bijux host versions change.
The default template values start new plugins at version `0.1.0` and use the current `cli_min` /
`cli_max` compatibility window from the template defaults.
For a pre-1.0 Bijux host, that window should stop at the next supported minor line instead of
claiming compatibility with every future `0.x` release.
Use `plugin_namespace` for the durable CLI name and compiled binary name, and use `crate_name` for
the Rust library identifier when `project_name` contains presentation-only punctuation.
If `project_name` includes leading digits or punctuation that should not survive into identifiers,
pass `project_slug`, `plugin_namespace`, and `crate_name` explicitly.
Cookiecutter validation rejects namespaces that do not start with a letter, crate names that are
not lowercase snake_case, invalid semver or inverted compatibility windows, and Rust keyword crate
names. It also blocks namespaces reserved by `bijux-cli` or official Bijux tools.
The generated Rust template stays local-development friendly by rebuilding the debug binary when the
wrapper detects source drift. Packaging a release binary later removes that local Cargo dependency
for distribution, but it is not required to validate the manifest contract during development.
