# API Usage

## Purpose

This guide explains the supported Python embedding surface without implying a
larger in-process API than the project currently guarantees.

## Scope

It covers the Python wrapper package and the small facade exported by
`bijux_cli_py`. It does not describe HTTP APIs or re-document the CLI command
syntax.

## Supported Python Surface

Use the Python package when you need to inspect version information, query
runtime metadata, or execute CLI argv from Python code.

Common entrypoints include:

- `version()`
- `command_tree_introspection()`
- `execution_facade(argv)`
- `execution_facade_with_status(argv)`

## Example

```python
from bijux_cli_py import execution_facade_with_status

result = execution_facade_with_status(["status", "--format", "json"])
if result.exit_code == 0:
    print(result.stdout)
else:
    print(result.stderr)
```

## Behavior Notes

- The Python facade targets the Rust runtime.
- Runtime discovery follows the same rules documented in the installation and
  migration guides.
- Callers stay responsible for interpreting stdout, stderr, and exit codes.
- Use the CLI when you need shell-oriented behavior such as output routing or
  process exit handling.

## References

- [Integrations and routed runtimes](../06-reference/integrations-and-routed-runtimes.md)
- [Python runtime migration](python/runtime-migration.md)
