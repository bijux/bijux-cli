# ADR 0002: Global Flag Precedence Contract

- **Date:** 2025‑08‑01
- **Status:** Accepted
- **Author:** Bijan Mousavi

---

## Context
Our Bijux CLI requires a deterministic and testable handling of global flags (`--help`, `--quiet`, `--debug`, `--format`, `--pretty`/`--no-pretty`, `--verbose`) across all commands to ensure consistency. This prevents ambiguous behaviors in hypothesis-driven fuzz tests, provides a clear "short-circuit" model for scripting and human users, and maintains synchronization between documentation, ADRs, and implementations. Without a formal contract, variations in flag-resolution rules could lead to edge cases and debugging challenges.

## Decision
All Bijux CLI commands must enforce global flags in the following strict precedence order, with exact semantics applied uniformly:

1. **Help** (`-h` / `--help`)
   * Short-circuits all other processing.
   * Immediately prints usage information and exits with code 0.
   * Skips validation or processing of any other flags or arguments.

2. **Quiet** (`-q` / `--quiet`)
   * Applies only if **help** is absent.
   * Suppresses all normal output on stdout and stderr.
   * Still performs full validation of flags and arguments, exiting with 0 on success or non-zero on errors.
   * Overrides **debug**, **format**, **pretty/no-pretty**, and **verbose** for output suppression, but not for exit codes.

3. **Debug** (`--debug`)
   * Applies only if neither **help** nor **quiet** is present.
   * Emits diagnostics and full trace information to stderr.
   * Implicitly enables **verbose** output (e.g., runtime metadata).
   * Forces `--pretty` formatting, overriding any `--no-pretty`.

4. **Format** (`-f <fmt>` / `--format <fmt>`)
   * Applies only if neither **help** nor **quiet** is present.
   * Requires a valid format name (`json` or `yaml`, case-insensitive).
   * Invalid or missing value triggers a structured error payload and exit code 2.

5. **Pretty** / **No-Pretty** (`--pretty` / `--no-pretty`)
   * Applies only if neither **help** nor **quiet** is present.
   * Controls indentation for human-readable structured output.
   * Defaults to `--pretty` if neither is specified.
   * Overridden by **debug**, which always enforces pretty formatting.

6. **Verbose** (`-v` / `--verbose`)
   * Applies only if neither **help** nor **quiet** is present.
   * Appends runtime metadata (e.g., Python version, platform) to structured output.
   * No-op under **quiet**; implied by **debug**.

### Error-Handling Rules
* Under **help**, always exit 0 with usage displayed, ignoring any invalid flags or arguments.
* Under **quiet**, suppress both stdout and stderr and return only an exit code (no JSON/YAML payload).
* Standard exit codes apply otherwise:
   * `0`: Success
   * `1`: Internal/fatal errors
   * `2`: Bad CLI usage (missing/invalid flags or arguments)
   * `3`: ASCII/encoding hygiene failures
* Every error payload (JSON or YAML) must include:
   * `"error"`: Human-readable message
   * `"code"`: Numeric exit code

## Consequences
### Pros
* Ensures deterministic behavior, eliminating flakiness in fuzz tests and user interactions.
* Provides a single source of truth for flag handling, simplifying documentation and maintenance.
* Enhances testability by allowing assertions on specific argv patterns (e.g., `-h` always yields usage and exit 0).

### Cons
* Requires each command's entrypoint to inspect `sys.argv` (or Typer/Click context) before parsing, adding initial implementation overhead.
* Contributors unfamiliar with the precedence must adapt, with no flexibility for command-specific variations.

## Enforcement
* No command implementation or pull request is accepted unless it fully adheres to this precedence contract.
* CI pipelines and reviewers must verify compliance through tests, rejecting any deviations in flag handling or semantics.
* This policy is binding and non-negotiable to maintain CLI consistency.
