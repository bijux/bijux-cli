# Evidence Roast Memo 2026-03-07

## What is still shallow

1. Too many perf scenarios remain transitional and non-blocking while consuming ownership and review overhead.
2. Compare evidence remains advisory and cannot be used as release-strength proof, yet still appears large in surface area.
3. Registry-level `release_blocking=true` breadth (74 assets) is wider than the enforced release blocking set (7 assets), which can confuse governance discussions.
4. Some battle and runtime contracts still verify existence/shape signals where stronger behavior checks should dominate.
5. Runtime scope still contains speculative families that are policy-labeled for move/delete and should not be treated as foundation proof.

## What would be fraudulent

- Reporting high test totals as release confidence without trust-property coverage mapping.
- Treating advisory compare/perf assets as blocking release evidence.
- Claiming battle-grade readiness without adversarial scenario outcomes in release summaries.
- Expanding taxonomy and metadata machinery while leaving weak battle assertions unpruned.

## Required next hardening moves

1. Expand release blocking set only with adversarial battle assets that close named trust gaps.
2. Demote or delete low-value perf/compare scenarios that do not protect a release trust boundary.
3. Keep runtime overreach cleanup active until speculative surfaces are moved out of runtime ownership.
4. Keep evidence growth frozen until this memo’s weaknesses have owners and dated closure plans.
