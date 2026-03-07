# History Retention Policy

## Retention baseline
- Keep authoritative run directories under the configured runs root.
- Retention is currently manual; no automatic deletion is applied by analytics commands.

## Operational guidance
- Keep enough history for flake and trend analysis windows used by your team.
- Prune old runs with explicit operator action outside analytics commands.

## Authority model
- Authoritative run data: run directory contents produced by execution.
- Derived analytics caches: optional, disposable, and recomputable.

## Corruption handling
- Corrupt or partial runs remain part of history but are surfaced as degraded inputs.
- Analytics commands should report partial views rather than mutating or repairing history.
