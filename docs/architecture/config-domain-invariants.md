# Config Domain Invariants

This document records invariants for the Rust config domain types.

## `ConfigKey`

- Always normalized to lowercase.
- Optional `BIJUXCLI_` prefix is removed during normalization.
- Must be ASCII.
- Must contain only alphanumeric characters and `_`.
- Must not be empty and must not contain `.`.

## `ConfigValue`

- Must be ASCII.
- Must not contain newline, tab, or vertical/form control characters.
- Stored as raw string after validation.

## `ConfigEntry`

- Always pairs a validated `ConfigKey` with a validated `ConfigValue`.

## `ConfigSnapshot`

- Represents one deterministic map of active entries keyed by normalized `ConfigKey`.

## `ConfigMutation`

- `Set` updates or creates one key.
- `Unset` targets one key.
- `Clear` removes all active entries.

## `ConfigSource` (read source)

- One of: `flags`, `env`, `file`, `defaults`.
- Used only to explain precedence outcome for resolved reads.

## `ResolvedConfigValue`

- Contains resolved key/value plus winning source.
- `source_path` is optional and present only when path context applies.

## `ConfigPathSet`

- Tracks active config/history/plugins paths used by one command execution.

## `ConfigLoadResult`

- Bundles loaded snapshot and resolved paths used for the load.

## `ConfigWriteResult`

- Captures whether write mutated state and final entry count.
- Always identifies write target path.

## `ConfigExportFormat`

- Allowed values: `env`, `json`, `yaml`.

## `ConfigCommandResult`

- Minimal command envelope for command status and command identity.

## `ConfigErrorKind`

- Allowed categories: `validation`, `parse`, `persistence`, `conflict`, `not-found`.

## `ConfigValidationError`

- Captures validation message and optional associated key.

## `ConfigParseError`

- Captures line number, line content, and parse error message.

## `ConfigPersistenceError`

- Captures path, operation, and persistence failure reason.

## `ConfigConflictError`

- Captures conflict message and optional conflicting key.

## `ConfigReloadResult`

- Captures reload status and source path.

## `ConfigClearResult`

- Captures clear status and removed key count.
