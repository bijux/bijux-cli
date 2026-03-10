# Parser Parity Audit

Date: 2026-03-09
Scope: `crates/bijux-cli/src/routing/parser.rs` versus current Python CLI documentation and command inventory.

## Findings

1. `parse_intent` currently swallows all clap parse failures and returns an empty intent.
- Impact: hard to distinguish usage errors vs help/version display paths in parser tests.
- Parity risk: medium.

2. Rust parser supports `--format text`, while Python global format contract is JSON/YAML-first.
- Impact: machine-output behavior mismatch risk.
- Parity risk: medium.

3. Rust root command tree is intentionally narrower than Python (`atlas`, `audit`, `docs`, `history`, `memory`, `sleep` are not yet routed in Rust).
- Impact: command availability mismatch.
- Parity risk: expected partial parity.

4. Legacy alias normalization is present for: `status`, `doctor`, `version`, `repl`, `completion`, `inspect`, `config get/set`, `plugins list/inspect`, `dev {routes,registry,env,doctor,contracts}`.
- Impact: compatibility coverage is good for currently ported route set.
- Parity risk: low for covered aliases.

5. Global flags are declared `global(true)` and are accepted before and after namespace tokens.
- Impact: placement parity mostly aligned with Python precedence surface.
- Parity risk: low.

6. Empty grouped commands (`bijux cli`, `bijux dev cli`) parse as grouped path without leaf action.
- Impact: runtime must map these to help/usage behavior consistently.
- Parity risk: medium.

7. No parser-level fixture corpus currently enforces parity for documented Python command inventory.
- Impact: drift risk.
- Parity risk: high.

## Remediation Added In This Batch
- Added Python-vs-Rust parse corpus artifacts and fixture files.
- Added fixture-driven parser tests for documented commands, aliases, invalid forms, and flag permutations.
- Added command-tree diff report comparing Python documented roots to Rust routed roots.
