# Evidence Topology Before And After Consolidation

Date: 2026-03-07

## Before

- Canonical proof assets were fragmented across root and crate islands.
- Legacy roots carried canonical scenarios: `examples/`, `benchmarks/`, `comparisons/`, `tests/e2e/fixtures`.
- Evidence governance was a registry overlay, not the sole proof pillar.

## After

- Canonical proof assets are rooted under `evidence/` with registry and ownership controls.
- Legacy canonical roots are removed from release-truth ownership.
- Consumers access governed assets through control-plane and testkit helper boundaries.

## Quantitative delta

| Indicator | Before | After |
| --- | ---: | ---: |
| evidence asset families | 5 | 8 |
| total evidence assets | n/a | 128 |
| release-blocking assets in registry | n/a | 74 |
| advisory assets in release set | n/a | 2 |
| canonical root scenario roots outside `evidence/` | many | 0 |

## Result

`evidence/` is now the authoritative proof pillar; test and crate surfaces are consumers, not owners.
