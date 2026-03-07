# Repository proof statement

## What this repository can prove today

- plan lowering determinism is enforced by planner contracts and battle trust linkage
- scheduler readiness and determinism invariants are enforced by scheduler contracts
- node and run state-machine legality is enforced by state-machine contracts
- backend lifecycle behavior is guarded by backend contract and parity surfaces
- run directory, import/export, and artifact integrity are guarded by hardening contracts
- cache correctness and corruption handling are guarded by cache contracts
- replay equivalence and divergence detection are guarded by replay contracts
- config precedence and policy determinism are guarded by config/policy contracts
- operator inspection outputs are guarded by inspection contracts and schemas

## Boundaries and non-claims

- this statement does not claim unimplemented distributed/container/batch production execution support
- release readiness requires evidence surfaces, not raw test totals

## Authority and governance

- evidence orchestration: `bijux-dev-dag` foundation and foundation-hardening suites
- trust property authority: `configs/policy/battle_trust_properties.json`
