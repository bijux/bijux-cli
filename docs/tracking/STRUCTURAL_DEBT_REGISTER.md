# Structural Debt Register

## Active debt items
1. Split very large command dispatch files in app and governance crates.
2. Continue reducing runtime/lib re-export concentration.
3. Replace include-based internal test aggregation where still present.
4. Track modeled informational types that can move out of runtime surfaces.

## Triage rule
Debt items are closed only with code change + contract test coverage + report update.
