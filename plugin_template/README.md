# {{cookiecutter.project\_name}}

A starter template for building **Bijux CLI** plugins—clean, typed, and ready to ship.

---

## Quick start

```bash
# 1) Scaffold from this template (run from your workspace)
mkdir -p ./tmp && cd ./tmp
bijux plugins scaffold my_plugin --template=../plugin_template --force

# 2) Install the new plugin (name inferred from folder basename)
cd ..
bijux plugins install ./tmp/my_plugin --force

# 3) Verify it’s healthy
bijux plugins check my_plugin
bijux plugins info my_plugin
bijux dev list-plugins
```

Uninstall anytime:

```bash
bijux plugins uninstall my_plugin
```

---

## What you get

* **Zero-boilerplate setup** for a Bijux plugin.
* **Typed hooks** and a minimal command to extend the CLI.
* **Structured output** (JSON/YAML) consistent with core flags.
* Clear spots to add docs, tests, and CI later.

> Once installed, plugins appear as **top-level commands** (or can register hooks to extend existing behavior).

---

## Develop & iterate

During development, just reinstall over the existing copy:

```bash
# Make changes to your plugin, then:
bijux plugins install ./tmp/my_plugin --force
bijux plugins check my_plugin
```

Tip: keep your plugin under version control and add tests early.

---

## Need more?

* List all installed plugins: `bijux plugins list`
* Show details: `bijux plugins info <name|path>`
* Validate structure/signature: `bijux plugins check <name|path>`

Build well. Break nothing.
