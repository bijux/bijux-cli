# Adapter Contract

This document is generated from runtime adapter descriptors and backend contract references.

## Scope

This contract governs the registered runtime adapter identities, the published
conformance scenario meanings, the external adapter handshake boundary, and the
backend-specific adapter mappings that the runtime treats as supported
integration surfaces.

## Registered adapters
- `const` `0.1`: kinds=["const"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `container` `0.1`: kinds=["container"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `file_transform` `0.1`: kinds=["file_transform"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `http` `0.1`: kinds=["http"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `python` `0.1`: kinds=["python"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `shell` `0.1`: kinds=["shell"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact

## Conformance scenarios
### const 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime completed const execution successfully
  observed status=success, failure_code=none, adapter=const@0.1, schema=v0.1, outputs=value
- `failure`: Skip (enforced_by_runtime=false, advisory_only=true, checked_by_execution=false) - const adapter has no runtime failure path for valid node definitions
- `missing_output`: Skip (enforced_by_runtime=false, advisory_only=true, checked_by_execution=false) - const adapter always materializes its declared value output
- `timeout`: Skip (enforced_by_runtime=false, advisory_only=true, checked_by_execution=false) - const adapter does not expose timeout-sensitive work
- `output_manifest`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime wrote outputs manifest for const with files value
  observed status=success, failure_code=none, adapter=const@0.1, schema=v0.1, outputs=value
- `failure_schema`: Skip (enforced_by_runtime=false, advisory_only=true, checked_by_execution=false) - const adapter does not emit structured failure payloads for successful value materialization
- `adapter_identity_schema`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - trace recorded adapter identity const@0.1 with schema v0.1
  observed status=success, failure_code=none, adapter=const@0.1, schema=v0.1, outputs=value

### container 0.1
- `success`: Skip (enforced_by_runtime=true, advisory_only=false, checked_by_execution=false) - container adapter conformance requires a repository-owned image fixture and remains intentionally skipped until that fixture is defined
- `failure`: Skip (enforced_by_runtime=true, advisory_only=false, checked_by_execution=false) - container adapter conformance requires a repository-owned image fixture and remains intentionally skipped until that fixture is defined
- `missing_output`: Skip (enforced_by_runtime=true, advisory_only=false, checked_by_execution=false) - container adapter conformance requires a repository-owned image fixture and remains intentionally skipped until that fixture is defined
- `timeout`: Skip (enforced_by_runtime=true, advisory_only=false, checked_by_execution=false) - container adapter conformance requires a repository-owned image fixture and remains intentionally skipped until that fixture is defined
- `output_manifest`: Skip (enforced_by_runtime=true, advisory_only=false, checked_by_execution=false) - container adapter conformance requires a repository-owned image fixture and remains intentionally skipped until that fixture is defined
- `failure_schema`: Skip (enforced_by_runtime=true, advisory_only=false, checked_by_execution=false) - container adapter conformance requires a repository-owned image fixture and remains intentionally skipped until that fixture is defined
- `adapter_identity_schema`: Skip (enforced_by_runtime=true, advisory_only=false, checked_by_execution=false) - container adapter conformance requires a repository-owned image fixture and remains intentionally skipped until that fixture is defined

### file_transform 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime completed file_transform execution successfully
  observed status=success, failure_code=none, adapter=file_transform@0.1, schema=v0.1, outputs=artifact
- `failure`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure EXEC_ERROR (user) for file_transform
  observed status=failed, failure_code=EXEC_ERROR, adapter=file_transform@0.1, schema=v0.1, outputs=none
- `missing_output`: Skip (enforced_by_runtime=false, advisory_only=true, checked_by_execution=false) - file_transform validates operation-specific output cardinality before generic runtime missing-output inspection
- `timeout`: Skip (enforced_by_runtime=true, advisory_only=true, checked_by_execution=false) - file_transform timeout coverage remains adapter-specific and is not emitted by the generic conformance harness
- `output_manifest`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime wrote outputs manifest for file_transform with files artifact
  observed status=success, failure_code=none, adapter=file_transform@0.1, schema=v0.1, outputs=artifact
- `failure_schema`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure EXEC_ERROR (user) for file_transform
  observed status=failed, failure_code=EXEC_ERROR, adapter=file_transform@0.1, schema=v0.1, outputs=none
- `adapter_identity_schema`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - trace recorded adapter identity file_transform@0.1 with schema v0.1
  observed status=success, failure_code=none, adapter=file_transform@0.1, schema=v0.1, outputs=artifact

### http 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime completed http execution successfully
  observed status=success, failure_code=none, adapter=http@0.1, schema=v0.1, outputs=response
- `failure`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure HTTP_STATUS_ERROR (execution) for http
  observed status=failed, failure_code=HTTP_STATUS_ERROR, adapter=http@0.1, schema=v0.1, outputs=response
- `missing_output`: Skip (enforced_by_runtime=false, advisory_only=true, checked_by_execution=false) - http adapter always materializes the response artifact before runtime output inspection
- `timeout`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure EXEC_TIMEOUT (timeout) for http
  observed status=failed, failure_code=EXEC_TIMEOUT, adapter=http@0.1, schema=v0.1, outputs=none
- `output_manifest`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime wrote outputs manifest for http with files response
  observed status=success, failure_code=none, adapter=http@0.1, schema=v0.1, outputs=response
- `failure_schema`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure HTTP_STATUS_ERROR (execution) for http
  observed status=failed, failure_code=HTTP_STATUS_ERROR, adapter=http@0.1, schema=v0.1, outputs=response
- `adapter_identity_schema`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - trace recorded adapter identity http@0.1 with schema v0.1
  observed status=success, failure_code=none, adapter=http@0.1, schema=v0.1, outputs=response

### python 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime completed python execution successfully
  observed status=success, failure_code=none, adapter=python@0.1, schema=v0.1, outputs=result
- `failure`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure PYTHON_EXCEPTION (execution) for python
  observed status=failed, failure_code=PYTHON_EXCEPTION, adapter=python@0.1, schema=v0.1, outputs=none
- `missing_output`: Skip (enforced_by_runtime=false, advisory_only=true, checked_by_execution=false) - python adapter failures are reported as structured execution exceptions before runtime output inspection
- `timeout`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure EXEC_TIMEOUT (timeout) for python
  observed status=failed, failure_code=EXEC_TIMEOUT, adapter=python@0.1, schema=v0.1, outputs=none
- `output_manifest`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime wrote outputs manifest for python with files result
  observed status=success, failure_code=none, adapter=python@0.1, schema=v0.1, outputs=result
- `failure_schema`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure PYTHON_EXCEPTION (execution) for python
  observed status=failed, failure_code=PYTHON_EXCEPTION, adapter=python@0.1, schema=v0.1, outputs=none
- `adapter_identity_schema`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - trace recorded adapter identity python@0.1 with schema v0.1
  observed status=success, failure_code=none, adapter=python@0.1, schema=v0.1, outputs=result

### shell 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime completed shell execution successfully
  observed status=success, failure_code=none, adapter=shell@0.1, schema=v0.1, outputs=value
- `failure`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure EXEC_FAIL (execution) for shell
  observed status=failed, failure_code=EXEC_FAIL, adapter=shell@0.1, schema=v0.1, outputs=none
- `missing_output`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure OUTPUT_MISSING (user) for shell
  observed status=failed, failure_code=OUTPUT_MISSING, adapter=shell@0.1, schema=v0.1, outputs=none
- `timeout`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure EXEC_TIMEOUT (timeout) for shell
  observed status=failed, failure_code=EXEC_TIMEOUT, adapter=shell@0.1, schema=v0.1, outputs=none
- `output_manifest`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime wrote outputs manifest for shell with files value
  observed status=success, failure_code=none, adapter=shell@0.1, schema=v0.1, outputs=value
- `failure_schema`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - runtime recorded structured failure EXEC_FAIL (execution) for shell
  observed status=failed, failure_code=EXEC_FAIL, adapter=shell@0.1, schema=v0.1, outputs=none
- `adapter_identity_schema`: Pass (enforced_by_runtime=true, advisory_only=false, checked_by_execution=true) - trace recorded adapter identity shell@0.1 with schema v0.1
  observed status=success, failure_code=none, adapter=shell@0.1, schema=v0.1, outputs=value

## External adapter protocol boundary
- `info --json` must emit machine JSON on stdout only.
- non-empty stderr during the info handshake is rejected.
- `execute` receives `--node-spec`, `--workdir`, `--outdir`, and `--failure-path`.
- nonzero adapter exits should write a `FailureInfo` JSON envelope to `--failure-path` for precise runtime failure mapping.
- external adapter binaries are fingerprinted into node trace evidence and cache identity.

## Slurm contract
- submit=`sbatch`, poll=`sacct`, cancel=`scancel`
- logs: streaming-chunked
- artifacts: stage outputs from scratch to run-dir artifact store

## Kubernetes contract
- namespace: `bijux-dag`
- job spec mapping: node resources and retry policy map into Job requests, limits, deadline, and backoff
- pod status mapping: terminal pod phases reconcile into runtime success/failure with retry classification
- logs: stdout/stderr streamed from pod logs and copied into node evidence
- artifacts: declared output files collected from mounted workdir after terminal pod state
- unsupported fields rejected: hostNetwork, hostPID, privileged, hostPath, runtimeClassName

## Fake batch executor
- submit=`submit(run_id,node_id) -> job_id`, poll=`snapshot(job_id) -> status`, cancel=`cancel(job_id, diagnostic)`
- states: queued, running, completed, failed, cancelled

## Versioning and change policy

Any incompatible change to registered adapter identities, conformance scenario
meanings, external adapter protocol fields, or backend contract mappings must
update this contract and the linked adapter tests in the same change.

## Related tests

- `crates/bijux-dag-runtime/tests/adapter_runtime_contracts.rs`
- `crates/bijux-dag-runtime/tests/adapter_backend_contracts.rs`
- `crates/bijux-dag-runtime/tests/adapter_reference_contracts.rs`
- `crates/bijux-dag-runtime/tests/adapter_sdk_contract.rs`
- `crates/bijux-dag-app/tests/adapter_command_contract.rs`
