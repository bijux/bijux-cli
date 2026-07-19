# Comparison Evidence

Use comparison evidence for executable side-by-side scenarios that explain
where Bijux matches another system and where it deliberately does not.

## What Lives Here

- `scenarios/`
- `baselines/`

## Scenario Policy

- Keep only scenarios with an executable bijux side and a committed baseline mapping.
- Treat narrative notes as docs, not comparison evidence, unless linked to executable scenario ids.
- Mark scenario metadata as `factual` or `descriptive`; prefer factual-only comparison assets.
- Every scenario must declare non-equivalence limits to prevent parity overclaiming.
- Comparison assets are non-release-blocking by default; release-blocking requires measured bijux evidence.

See `CONTRACT.md` for comparison-specific enforcement rules.
