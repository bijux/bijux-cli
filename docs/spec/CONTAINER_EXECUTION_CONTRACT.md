# Container Execution Contract

## Scope
This contract defines the minimum container execution model for `bijux-dag`.
Container execution is modeled as a backend contract surface. It is not an
adapter payload format.

## Required fields
- `image`: immutable image reference or digest
- `command`: argv vector
- `env`: explicit environment map after policy shaping
- `mounts`: local-to-container path mappings
- `declared_outputs`: container-relative output paths
- `timeout_ms`: optional execution timeout

## Path model
- Local artifact roots are mounted into a declared container root.
- Output paths must be normalized and remain under declared output root.
- Traversal (`..`) and absolute host path escape are rejected.

## Output declaration model
- A container node is successful only when declared outputs are present
  relative to declared output roots.
- Missing declared outputs is a contract violation.

## Error mapping
- missing image -> backend preparation error
- launch failure -> backend launch error
- timeout -> execution timeout classification
- missing outputs -> artifact contract error

## Environment isolation model
- `clean_env` applies before mount/launch assembly.
- Allowlist and denylist patterns are applied to all final environment keys.
- Undeclared or denied keys are removed.

## Kubernetes scope
Kubernetes execution is not implemented as a runnable backend in this repo.
Kubernetes material is limited to contract/model definitions and simulation.

## Versioning and compatibility
- Additive field additions are backward-compatible.
- Required-field semantic changes require contract version bump and tests.

## Verifying tests
- `crates/bijux-dag-runtime/tests/container_execution_contracts.rs`
- `crates/bijux-dag-runtime/tests/execution_backend_contract.rs`
