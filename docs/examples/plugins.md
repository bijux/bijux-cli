# Plugin Examples

## Purpose
These examples show the current plugin lifecycle contract used by `bijux-cli`.

## Example 1: Install a Local Plugin Manifest

Create a minimal local plugin directory:

```bash
mkdir -p ./my-plugin
cat > ./my-plugin/plugin.manifest.json <<'JSON'
{
  "name": "my-plugin",
  "version": "0.1.0",
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {
    "min_inclusive": "0.2.1-dev",
    "max_exclusive": "1.0.0"
  },
  "namespace": "my-plugin",
  "kind": "python",
  "aliases": [],
  "entrypoint": "plugin:main",
  "capabilities": []
}
JSON
cat > ./my-plugin/plugin.py <<'PY'
def main(argv: list[str]) -> dict[str, object]:
    return {"status": "ok", "argv": argv}
PY
```

Install it:

```bash
bijux plugins install ./my-plugin/plugin.manifest.json
```

If you install without overriding `--source`, the manifest path is used as the displayed source
label. Regardless of the displayed label, local manifest installs keep a manifest anchor so later
health checks can validate delegated entrypoints against the installed source tree.

## Example 2: Inspect and Check Installed Plugins

```bash
bijux plugins list
bijux plugins inspect my-plugin
bijux plugins check my-plugin
bijux plugins explain my-plugin
bijux plugins schema
```

Expected behavior:
- `list` includes `my-plugin`
- `inspect` shows manifest, source, and trust metadata
- `check` verifies manifest validity and current entrypoint presence
- `explain` shows compatibility or load-time diagnostics for the namespace

## Example 3: Uninstall and Verify Cleanup

```bash
bijux plugins uninstall my-plugin
bijux plugins list
```

After uninstall, `list` no longer reports the namespace and the registry stays consistent.
