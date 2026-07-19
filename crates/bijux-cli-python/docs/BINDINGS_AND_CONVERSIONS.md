# Bindings And Conversions

The bridge translates representation across Rust, JSON, PyO3, and Python. It
must preserve command meaning, streams, exit status, and failure category.

## Binding Surface

The native module exports these capability groups:

| Capability | Rust authority |
| --- | --- |
| runtime version | `version_binding_api` |
| command inspection | `command_tree_introspection_api` |
| successful execution facade | `execution_facade_api` |
| complete execution outcome | `execution_outcome_api` |
| install and config paths | install/config resolution helpers |
| plugin registry inspection | `plugin_registry_inspection_api` |

Convenience bindings for doctor, status, plugin listing, and REPL bootstrap
delegate through the same execution facade rather than implementing commands.

## Argument Contract

`normalized_argv` prepends `bijux` only when the caller did not provide it.
This keeps embedded calls aligned with `run_app`, whose parser expects a
process-style argument vector. Bindings must not rewrite command aliases,
global flags, or route segments; the Rust runtime owns normalization.

## Outcome Shapes

`execution_facade_api` is success-oriented. It returns stdout only for a zero
exit status and raises a classified bridge error otherwise.

`execution_outcome_api` is diagnostic-oriented. It always returns JSON with:

- `exit_code`;
- complete `stdout`;
- complete `stderr`;
- an optional `error_kind`.

Use the outcome API when a caller must inspect failures without losing stream
context. Do not infer success from non-empty stdout.

## Error Classification

`BridgeErrorKind` has three stable categories:

- `Usage` for parsing, namespace, and route errors;
- `Validation` for encoding and invalid-data failures;
- `Internal` for runtime failures not covered above.

Exit-code authority takes precedence over message inspection. Text matching is
a compatibility fallback for errors that do not carry an owned category.
Python exception tags are `UsageError`, `ValidationError`, and
`InternalError`.

The bridge must not collapse:

- non-zero command status into a successful return;
- invalid JSON into an empty object;
- subprocess failure into a validation error;
- stderr into stdout;
- unknown failures into usage errors.

## JSON And Schema Rules

Bridge payloads use deterministic `serde_json` serialization. Command
envelopes remain owned by `bijux-cli`; bridge-specific diagnostic wrappers are
not replacements for those schemas.

Plugin registry inspection validates JSON when a file exists. A missing
registry has a governed empty representation, while malformed registry content
is an error with path context.

## Verification

Focused Rust evidence:

```bash
cargo test --locked -p bijux-cli-python \
  --test bridge_bindings \
  --test bridge_conversion_contracts \
  --test bridge_execution_parity
```

Additional law and replay tests cover minimized conversion cases, stable
classification, binary parity, and surface parity. Python parity tests verify
that the facade exposes the same observable result with and without the native
extension.
