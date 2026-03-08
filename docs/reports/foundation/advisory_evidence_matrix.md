# Advisory Evidence Matrix

Source of truth:
- `configs/policy/evidence_command_classification.json`
- `configs/policy/release_evidence_policy.json`

| Evidence family | Verify command | Fast lane blocking by default | Full lane blocking |
| --- | --- | --- | --- |
| compare | `verify evidence-compare` | no | no |

Advisory evidence checks remain non-blocking unless explicitly enabled by operator policy.
