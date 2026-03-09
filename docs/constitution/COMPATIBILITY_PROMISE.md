# Compatibility Promise

## Purpose
Define what `bijux-cli` guarantees across versions and implementations.

## Scope
This document covers behavior contracts, migration expectations, install compatibility, and release strategy.

## Core Concepts
- Contractual behavior must remain stable in a compatibility window.
- Incidental behavior may evolve without compatibility guarantees.
- Compatibility is enforced for users, CI automation, and plugin ecosystems.

## Contractual Versus Incidental
### Contractual behaviors
- Root grammar and reserved namespaces in `CLI_CONSTITUTION.md`.
- Public global flags and precedence rules.
- Documented exit code mappings.
- Documented output and error envelope shape and routing rules.
- Documented REPL command availability and contract-level behavior.
- Documented plugin namespace and lifecycle compatibility rules.

### Incidental behaviors
- Exact non-contract debug wording.
- Non-documented whitespace and display formatting in text mode.
- Internal module boundaries and runtime architecture.

## Compatibility Window
- The project maintains compatibility for all documented contracts within a major version.
- For old Python package installs, supported compatibility window is the latest two minor releases in the current major line.
- Security or critical correctness fixes may narrow support for unmaintained old minors with notice in release notes.

## Install and Release Strategy
- If a Rust foundation introduces no documented contract breakage, ship as a minor release on PyPI.
- If a documented contract changes incompatibly, ship as a major release.
- `pip install bijux-cli` remains a supported install path.
- The package must continue to expose the `bijux` entrypoint.

## Migration Commitments
### Users who know only `pip install bijux-cli`
- Keep installation and binary invocation unchanged.
- Provide release-note migration guidance when behavior expands.

### Users expecting built-in command continuity
- Maintain existing built-in command paths or provide documented aliases.
- When command retirement is necessary, apply deprecation policy before removal.

### Users invoking `bijux` in CI
- Preserve exit-code meanings and machine-readable output contracts.
- Announce any CI-relevant change before the release that enforces it.

### Shell scripts consuming JSON/YAML output
- Preserve envelope top-level keys and semantic meanings.
- Keep `--format json --no-pretty` machine-safe and compact.
- Keep YAML and JSON payloads semantically equivalent for the same command.

### Automation relying on exit codes
- Keep the code-to-meaning mapping stable for documented failure categories.
- Treat any unplanned exit-code change as a compatibility regression.

### Existing plugin users
- Preserve plugin discovery semantics for `entrypoint` and local directory plugins.
- Preserve metadata compatibility validation and stable plugin error categories.
- Reserve built-in namespaces to prevent plugin route ambiguity.

### REPL users
- Preserve command parity and command parsing behavior documented in `REPL_PARITY.md`.
- Preserve stable shortcuts (`exit`, `quit`) and command chaining semantics.

## Legacy Command Continuity Decisions
These decisions apply when introducing `bijux cli ...` and `bijux dev cli ...` surfaces.

| Existing form | Compatibility decision | Long-term state |
| --- | --- | --- |
| `bijux status` | Keep as alias to `bijux cli status` | Supported alias |
| `bijux plugins ...` | Keep as alias to `bijux cli plugins ...` | Supported alias |
| `bijux config ...` | Keep as alias to `bijux cli config ...` | Supported alias |
| `bijux doctor` | Keep as alias to `bijux cli doctor` | Supported alias |
| `bijux version` | Keep as alias to `bijux cli version` | Supported alias |
| `bijux dev ...` | Keep as alias to `bijux dev cli ...` | Supported alias |

Commands are not removed without deprecation notice under `DEPRECATION_POLICY.md`.

## Deprecation Notice Contract
Deprecation messages use the template defined in `DEPRECATION_POLICY.md`:

`DEPRECATED: <subject> is deprecated and will be removed in <version>. Use <replacement>. Reference: <url>.`

## Dual Install Policy
- Dual installs (for example pip plus cargo) are supported only when they resolve to the same major version contract.
- `bijux doctor` must surface path resolution and version details to make conflicts diagnosable.
- When multiple binaries exist, the first resolved binary on `PATH` is authoritative for that invocation.

## Failure Modes
- Behavioral drift from contractual guarantees is a compatibility bug.
- Unannounced incompatible contract changes are release policy violations.

## Design Rationale
- Explicit contracts reduce breakage in scripts, CI, and plugin ecosystems.
- Compatibility windows keep maintenance practical while preserving user trust.

## Non-Goals
- Guaranteeing all undocumented behavior forever.
- Supporting every historical pre-release behavior.
