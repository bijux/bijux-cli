# Evidence to Suite Exercise Mapping Report

Mapping of evidence verify surfaces to execution paths.

| Verify command | Suite id | Severity | `make test-release` | `make evidence-all` |
| --- | --- | --- | --- | --- |
| `verify evidence-battle` | `evidence-battle` | release-critical | yes | yes |
| `verify evidence-cache` | `evidence-cache` | release-critical | yes | yes |
| `verify evidence-replay` | `evidence-replay` | release-critical | yes | yes |
| `verify evidence-compat` | `evidence-compat` | release-critical | yes | yes |
| `verify evidence-fault` | `evidence-fault` | release-critical | yes | yes |
| `verify evidence-perf` | `evidence-perf` | release-critical | yes | yes |
| `verify evidence-consumers` | `evidence-consumers` | release-critical | yes | yes |
| `verify evidence-release-set` | `evidence-release-set` | release-critical | yes | yes |
| `verify evidence-schema` | `evidence-schema` | release-supporting | no | yes |
| `verify evidence-registry` | `evidence-registry` | release-supporting | no | yes |
| `verify evidence-authoring` | `evidence-authoring` | release-supporting | no | yes |
| `verify evidence-drift` | `evidence-drift` | release-supporting | no | yes |
| `verify evidence-foundation` | `evidence-foundation` | release-supporting | no | yes |
| `verify evidence-compare` | `evidence-compare` | advisory | no | yes |

Sources:
- `configs/policy/evidence_rationalization_policy.json`
- `make/root.mk`
- `make/evidence.mk`
