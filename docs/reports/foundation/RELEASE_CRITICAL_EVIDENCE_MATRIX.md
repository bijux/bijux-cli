# Release-Critical Evidence Matrix

Source of truth:
- `configs/policy/evidence_command_classification.json`
- `evidence/release/release_evidence_set.json`

| Evidence family | Verify command | `make test-all` blocking | Release set required family |
| --- | --- | --- | --- |
| battle | `verify evidence-battle` | yes | yes |
| cache | `verify evidence-cache` | yes | yes |
| replay | `verify evidence-replay` | yes | implicit (`cache_replay`) |
| compat | `verify evidence-compat` | yes | yes |
| fault | `verify evidence-fault` | yes | yes |
| perf | `verify evidence-perf` | yes | yes |
| consumers | `verify evidence-consumers` | yes | governance gate |
| release-set | `verify evidence-release-set` | yes | governance gate |

Release-critical evidence failures are release-blocking in the full lane.
