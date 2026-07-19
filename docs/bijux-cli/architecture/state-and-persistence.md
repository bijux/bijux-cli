---
title: State and Persistence
audience: mixed
type: explanation
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# State and Persistence

`bijux-cli` owns local user state for configuration, command history, memory,
and installed plugin metadata. Path selection is part of command behavior:
two invocations using different state roots are not equivalent even when argv
is identical.

## State Topology

With no overrides, state is rooted under the effective home directory:

| State | Default path | Format and purpose |
| --- | --- | --- |
| configuration | `~/.bijux/.env` | validated key/value configuration and path settings |
| history | `~/.bijux/.history` | bounded JSON history entries |
| memory | `~/.bijux/.memory.json` | JSON object of explicit memory keys and values |
| plugin content | `~/.bijux/.plugins/` | installed manifests and entrypoints |
| plugin registry | `~/.bijux/.plugins/registry.json` | versioned installed-plugin index |

The memory file is derived from the selected configuration file's parent. It
does not have an independent path override.

## Path Resolution

Configuration, history, and plugin paths use strict precedence:

1. an applicable command-line path override;
2. `BIJUXCLI_CONFIG`, `BIJUXCLI_HISTORY_FILE`, or
   `BIJUXCLI_PLUGINS_DIR`;
3. values read from the compatibility configuration;
4. defaults under the effective home directory.

The current global CLI path flag overrides the configuration path. History and
plugin paths are selected through environment, configuration, or defaults.
Tilde-prefixed configured paths resolve against the effective home.

Home resolution uses `HOME`, then `USERPROFILE`, then
`HOMEDRIVE`/`HOMEPATH`. If none is available, the current directory is used
with a diagnostic warning. A malformed compatibility file is not silently
trusted: supported parse failures fall back to defaults and remain visible in
state diagnostics; underlying I/O failures still fail resolution.

## Read Semantics

Missing config, history, memory, or registry files represent empty or default
state where the owning repository permits it. Existing malformed state does
not:

- malformed configuration reports its parsing or validation error;
- history enforces a 16 MiB read limit, rejects a malformed top-level
  document, and reports invalid or truncated entries;
- memory must be a JSON object;
- the plugin registry must parse and carry the supported registry version.

Diagnostics report the resolved paths, file kind, size, readability,
writability, compatibility warnings, and detected corruption. They do not
repair state merely by reading it.

## Write Guarantees

Configuration, history, memory, compatibility settings, and plugin registry
serialization use a temporary file in the destination directory, flush file
contents, replace the destination, and sync the parent directory where the
platform supports it. This prevents readers from observing a partially written
document after a successful replacement.

Those guarantees are per file. There is no repository-wide transaction across
configuration, history, memory, and plugins.

Plugin registry mutation adds a stronger local protocol:

1. acquire an exclusive registry lock;
2. identify and remove a stale lock only under the governed timeout and process
   rules;
3. back up the current registry;
4. load, validate, and mutate the in-memory registry;
5. save atomically;
6. restore the backup if mutation or persistence fails;
7. remove the backup after success and release the lock.

The plugin lock coordinates registry writers. It does not sandbox plugin code
or serialize unrelated state files.

## Recovery Boundaries

Do not delete state as the first response to corruption. Preserve the failing
file and resolved-path diagnostics, then use the owning doctor or explicit
repair command.

- A malformed history entry may be dropped and counted when the surrounding
  array is readable; a malformed document remains an error.
- Forced history clearing can replace unreadable history, and reports that the
  corruption was ignored.
- Plugin recovery may quarantine invalid content or restore registry backup
  state through explicit operations.
- A stale lock is removable only after its owner or timeout state is
  established.

State repair changes user data. It must remain an explicit command outcome,
not an automatic side effect of help, route lookup, or diagnostics.

## Concurrency And Durability Limits

- Atomic replacement prevents partial single-file writes; it does not merge
  concurrent read-modify-write operations.
- The plugin registry has an owned writer lock; config, history, and memory do
  not share that lock.
- Parent-directory sync is implemented on Unix and is a no-op on other
  platforms.
- Filesystem, power-loss, network-home, and platform rename guarantees remain
  those of the host filesystem.

Callers requiring stronger coordination must provide it outside these
per-file APIs rather than inferring a global transaction.

## Implementation Anchors

- `crates/bijux-cli/src/features/diagnostics/state_paths.rs`
- `crates/bijux-cli/src/features/install/compatibility.rs`
- `crates/bijux-cli/src/features/config/operations.rs`
- `crates/bijux-cli/src/features/history/operations.rs`
- `crates/bijux-cli/src/features/memory/operations.rs`
- `crates/bijux-cli/src/features/plugins/registry.rs`
- `crates/bijux-cli/src/infrastructure/fs_store.rs`
- `crates/bijux-cli/src/infrastructure/state_store.rs`
- `crates/bijux-cli/tests/integration/cli/resilience/`

## Related Operations

- [Configuration Surface](../interfaces/configuration-surface.md)
- [Diagnostics Guide](../operations/diagnostics-guide.md)
- [Failure Recovery](../operations/failure-recovery.md)
- [Known Limitations](../quality/known-limitations.md)
