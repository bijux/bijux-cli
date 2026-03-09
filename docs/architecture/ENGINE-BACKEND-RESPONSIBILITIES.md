# Engine and Backend Responsibilities

## Side-by-side ownership

| Surface | Engine | Backend |
| --- | --- | --- |
| scheduling and readiness | yes | no |
| retry policy and attempt sequencing | yes | no |
| command/process/container launch internals | no | yes |
| stream capture internals | no | yes |
| run finalization | yes | no |
| backend cleanup strategy | no | yes |

## Boundary notes

- Engine executes plans and lifecycle orchestration.
- Backend implementations execute effects under capability constraints.
- Capability mismatch is a binding error.
