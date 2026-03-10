# CLI Constitution

## Purpose
Define the canonical command identity, namespace grammar, and ownership boundaries for `bijux-cli`.

## Scope
This document governs root command identity and namespace contracts.

## Core Concepts
- `bijux-cli` is the sole owner of the `bijux` binary.
- Root grammar is explicit and frozen for compatibility.
- Namespace reservations prevent collisions across products and plugins.

## Invariants
- The canonical binary name is `bijux`.
- `bijux-cli` is the only distribution that may claim canonical ownership of `bijux`.
- The root grammar includes these stable entry forms:
  - `bijux`
  - `bijux cli`
  - `bijux dev cli`
- Reserved root namespaces are:
  - `agent`
  - `atlas`
  - `cli`
  - `dev`
  - `dag`
  - `dna`
  - `gnss`
  - `help`
  - `version`
  - `doctor`
  - `rag`
  - `rar`
  - `repl`
  - `plugins`
  - `completion`
  - `inspect`
  - `vex`
  - `audit`
  - `config`
  - `docs`
  - `history`
  - `memory`
  - `sleep`
  - `status`

## Namespace Governance
### Reservation process for official Bijux products
1. A product owner proposes a namespace with intent, expected lifetime, and command surface.
2. Maintainers evaluate conflicts with existing root and plugin namespaces.
3. On approval, maintainers add the namespace to `KNOWN_BIJUX_TOOLS` and `official_product_namespace_registry.json`.
4. The reservation becomes contractual only after release.

### Plugin namespace rejection rules
- Plugins must not register any reserved root namespace.
- Plugins must not shadow built-in command paths.
- Plugins must not differ only by case from existing namespaces.
- Rejected namespaces return a stable validation failure with guidance.

### Naming normalization rules
- Public namespace matching is case-insensitive at parse time.
- Canonical persisted names are lowercase kebab-case.
- Multiple separators normalize to a single `-`.
- Leading and trailing separators are stripped before validation.

### Public namespace style
- Public command namespaces use `kebab-case`.
- `snake_case` remains unsupported for new public namespaces.

## Failure Modes
- Namespace conflict or reserved-name usage is a user validation error.
- Binaries that claim `bijux` while diverging from this constitution are non-compliant.

## Design Rationale
- Single-binary ownership prevents ambiguous operational behavior.
- Frozen grammar enables stable shell scripts and CI workflows.
- Kebab-case aligns with common CLI conventions and readability.

## Non-Goals
- Internal parser implementation details.
- Plugin runtime behavior beyond namespace rules.
