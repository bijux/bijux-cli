---
title: Failure Recovery
audience: mixed
type: guide
status: canonical
owner: bijux-cli-docs
last_reviewed: 2026-07-19
---

# Failure Recovery

Recovery is complete when the cause is understood, the smallest owned surface
has been repaired, and the original command succeeds under the same inputs.
Deleting the entire state directory may make a symptom disappear, but it
destroys the evidence needed to establish either cause or correctness.

## Preserve Before Diagnosing

Start with information that does not require a broad health scan:

```bash
bijux cli paths --format json --no-pretty
bijux version --format json --no-pretty
```

Record the original command, stdout, stderr, exit code, current directory,
relevant environment overrides, and resolved paths. Before running a repair or
a broad diagnostic command, preserve the existing configuration, history,
memory, plugin registry, and adjacent `.bak` or `.tmp` files under an
access-controlled recovery directory.

State snapshots can contain commands, paths, plugin provenance, and secrets.
Do not attach them to an issue or support request without inspection and
redaction.

!!! warning "Broad diagnostics can repair plugin state"
    `bijux status`, `bijux audit`, `bijux doctor`, and
    `bijux doctor paths` evaluate state diagnostics. When the plugin registry
    is corrupt, that evaluation can quarantine the registry, write an empty
    replacement, and remove a stale registry `.bak` file. Capture the original
    files first when plugin-registry corruption is possible.

`bijux doctor --bundle` writes a useful report under
`artifacts/bijux-cli/doctor-bundle`, but it is not a raw-state backup. It
records the post-diagnostic observation and generated references.

## Classify The Failure

| Symptom | Likely owner | First evidence | Avoid |
| --- | --- | --- | --- |
| exit `2`, unknown option, missing argument | parser or validation | structured error and command help | changing state |
| wrong config value | configuration precedence | `config explain`, `config diff`, `config validate` | editing every config source |
| unreadable or unexpected path | path resolution or permissions | `cli paths`, then a preserved-state `doctor paths` | moving files before recording resolved paths |
| corrupt plugin registry | plugin registry | preserved registry and `plugins doctor` result | assuming doctor is read-only |
| healthy registry but failing plugin | plugin process or compatibility | `plugins inspect`, `plugins check`, `plugins explain` | deleting unrelated plugins |
| malformed history | history state | preserved history and `history --format json` | `history clear --force` before backup |
| malformed memory | memory state | preserved memory file and `memory list` error | expecting `memory clear` to bypass parsing |
| wrong executable or duplicate install | installation | `version`, `cli paths`, `doctor shims` | reinstalling before identifying the active binary |
| mounted app failure | owning product runtime | `apps which`, `apps version`, `apps doctor NAME` | treating product behavior as a CLI state defect |

Parser and validation failures do not justify a state mutation. Correct the
invocation first. Runtime failures should be routed to the component named by
the error envelope and route ownership.

## Recover Configuration

Use read-only resolution before repair:

```bash
bijux config explain KEY --format json --no-pretty
bijux config validate --format json --no-pretty
```

If the global config is malformed:

```bash
bijux config repair --format json --no-pretty
bijux config validate --format json --no-pretty
```

`config repair` is intentionally lossy for invalid lines. It parses supported
entries, drops invalid values and malformed rows, writes the original content
to the sibling `.bak` path when a change is required, and atomically writes the
sanitized file. Retain the returned `issues`, `remediation`,
`dropped_line_count`, and `backup` fields. Do not delete the backup until the
repaired values and original workload have been verified.

For precedence conflicts rather than corruption, change the highest source
that is actually wrong. A repair of the global file cannot override a
conflicting environment variable, project file, profile, or command-line
override.

## Recover Plugins

For a readable registry, isolate one plugin:

```bash
bijux plugins inspect NAME --format json --no-pretty
bijux plugins check NAME --format json --no-pretty
bijux plugins explain NAME --format json --no-pretty
bijux plugins disable NAME --format json --no-pretty
```

Disabling is preferable to uninstalling during diagnosis because the registry
retains identity and provenance while preventing normal execution.

For a corrupt registry, preserve the registry and sibling transaction files
before running:

```bash
bijux plugins doctor --format json --no-pretty
```

The doctor quarantines the corrupt file using a
`registry.corrupt-<timestamp>.json` name and creates an empty active registry.
The report is degraded even when that self-repair succeeds. Reconstruct active
entries from trusted manifests and provenance; do not treat an empty registry
as proof that installed plugin files are safe or absent.

## Recover History And Memory

History accepts the current JSON-array format and bounded legacy line formats.
A malformed JSON object, invalid JSON array, oversized file, broken symlink, or
invalid UTF-8 fails closed. After preserving the original:

```bash
bijux history clear --force --format json --no-pretty
```

Forced clear reports `corruption_ignored` and the original read error, then
writes an empty history. It does not create a backup itself.

Memory has no forced repair command. Every mutation reads the existing JSON
object first, so `memory clear` also fails on malformed memory. Preserve the
file, move the malformed copy out of the active path, allow the runtime to
create a new object, and restore only reviewed keys through `memory set
KEY=VALUE`. Keep the original until the restored state has been verified.

## Recover Installation And Delegation

Establish which executable owns the failure:

```bash
bijux cli paths --format json --no-pretty
bijux doctor shims --format json --no-pretty
bijux apps which NAME --format json --no-pretty
bijux apps version NAME --format json --no-pretty
```

Resolve `PATH` shadowing or duplicate installations before changing product
state. For an official product, use `apps doctor NAME`; for a plugin, use the
plugin diagnostics. The general CLI can report delegation and compatibility
facts, but it cannot repair behavior owned by another executable.

## Prove Recovery

Recovery evidence should show:

1. the original failure and stable error category
2. the exact state path and pre-repair copy
3. one bounded remediation and its structured result
4. the same diagnostic after remediation
5. the original command succeeding with the same relevant inputs

If the command succeeds only after broad deletion, the cause remains unknown.
Restore the preserved state in an isolated environment and narrow the failing
domain before claiming resolution.

## Implementation Authorities

- path resolution and state diagnostics:
  `crates/bijux-cli/src/features/diagnostics/state_paths.rs`
- configuration repair:
  `crates/bijux-cli/src/features/config/layered.rs`
- plugin quarantine and registry rollback:
  `crates/bijux-cli/src/features/plugins/diagnostics.rs` and
  `crates/bijux-cli/src/features/plugins/registry.rs`
- history and memory storage:
  `crates/bijux-cli/src/infrastructure/state_store.rs`
- diagnostic bundle:
  `crates/bijux-cli/src/interface/cli/handlers/cli.rs`

## Related Operations

- [Diagnostics Guide](diagnostics-guide.md)
- [Configuration Surface](../interfaces/configuration-surface.md)
- [Operator Workflows](../interfaces/operator-workflows.md)
- [Security and Safety](security-and-safety.md)
- [Risk Register](../quality/risk-register.md)
