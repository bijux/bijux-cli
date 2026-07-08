# Adapter Contract

This document is generated from runtime adapter descriptors and backend contract references.

## Registered adapters
- `const` `0.1`: kinds=["const"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `container` `0.1`: kinds=["container"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `file_transform` `0.1`: kinds=["file_transform"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `http` `0.1`: kinds=["http"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `python` `0.1`: kinds=["python"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact
- `shell` `0.1`: kinds=["shell"], origin=BuiltIn, schema=v0.1, timeout=true, cancel=false, cache=FingerprintExact

## Conformance scenarios
### const 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false) - successful adapter execution is a required runtime path
- `failure`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not expose a process failure boundary
- `argv_contract`: Skip (enforced_by_runtime=false, advisory_only=true) - argv validation is specific to shell-backed command adapters
- `missing_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime validates declared output files for every adapter execution
- `undeclared_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime rejects files written outside the declared output contract
- `timeout`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime enforces declared timeout budgets and records timeout failures
- `cancel`: Skip (enforced_by_runtime=false, advisory_only=true) - adapter descriptor does not declare cancellation support
- `env_policy`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not read process environment directly
- `workdir_isolation`: Skip (enforced_by_runtime=false, advisory_only=true) - in-process adapters do not cross a working-directory boundary
- `missing_executable`: Skip (enforced_by_runtime=false, advisory_only=true) - in-process adapters do not resolve external executables
- `cache_output`: Pass (enforced_by_runtime=true, advisory_only=false) - produced output schema matches the expected adapter schema
- `large_stdout`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not emit stdout streams
- `non_utf8_output`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not emit process byte streams

### container 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false) - successful adapter execution is a required runtime path
- `failure`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime records explicit execution failure results
- `argv_contract`: Skip (enforced_by_runtime=false, advisory_only=true) - argv validation is specific to shell-backed command adapters
- `missing_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime validates declared output files for every adapter execution
- `undeclared_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime rejects files written outside the declared output contract
- `timeout`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime enforces declared timeout budgets and records timeout failures
- `cancel`: Skip (enforced_by_runtime=false, advisory_only=true) - adapter descriptor does not declare cancellation support
- `env_policy`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime shapes and filters adapter environments before execution
- `workdir_isolation`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime executes process-backed adapters from a dedicated node work directory
- `missing_executable`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime reports executable resolution failures with structured infrastructure errors
- `cache_output`: Pass (enforced_by_runtime=true, advisory_only=false) - produced output schema matches the expected adapter schema
- `large_stdout`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime captures stdout and stderr as node evidence files
- `non_utf8_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime stores output bytes and artifact files without requiring UTF-8 payloads

### file_transform 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false) - successful adapter execution is a required runtime path
- `failure`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime records explicit execution failure results
- `argv_contract`: Skip (enforced_by_runtime=false, advisory_only=true) - argv validation is specific to shell-backed command adapters
- `missing_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime validates declared output files for every adapter execution
- `undeclared_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime rejects files written outside the declared output contract
- `timeout`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime enforces declared timeout budgets and records timeout failures
- `cancel`: Skip (enforced_by_runtime=false, advisory_only=true) - adapter descriptor does not declare cancellation support
- `env_policy`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not read process environment directly
- `workdir_isolation`: Skip (enforced_by_runtime=false, advisory_only=true) - in-process adapters do not cross a working-directory boundary
- `missing_executable`: Skip (enforced_by_runtime=false, advisory_only=true) - in-process adapters do not resolve external executables
- `cache_output`: Pass (enforced_by_runtime=true, advisory_only=false) - produced output schema matches the expected adapter schema
- `large_stdout`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not emit stdout streams
- `non_utf8_output`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not emit process byte streams

### http 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false) - successful adapter execution is a required runtime path
- `failure`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime records explicit execution failure results
- `argv_contract`: Skip (enforced_by_runtime=false, advisory_only=true) - argv validation is specific to shell-backed command adapters
- `missing_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime validates declared output files for every adapter execution
- `undeclared_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime rejects files written outside the declared output contract
- `timeout`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime enforces declared timeout budgets and records timeout failures
- `cancel`: Skip (enforced_by_runtime=false, advisory_only=true) - adapter descriptor does not declare cancellation support
- `env_policy`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not read process environment directly
- `workdir_isolation`: Skip (enforced_by_runtime=false, advisory_only=true) - in-process adapters do not cross a working-directory boundary
- `missing_executable`: Skip (enforced_by_runtime=false, advisory_only=true) - in-process adapters do not resolve external executables
- `cache_output`: Pass (enforced_by_runtime=true, advisory_only=false) - produced output schema matches the expected adapter schema
- `large_stdout`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not emit stdout streams
- `non_utf8_output`: Skip (enforced_by_runtime=false, advisory_only=true) - non-process adapters do not emit process byte streams

### python 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false) - successful adapter execution is a required runtime path
- `failure`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime records explicit execution failure results
- `argv_contract`: Skip (enforced_by_runtime=false, advisory_only=true) - argv validation is specific to shell-backed command adapters
- `missing_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime validates declared output files for every adapter execution
- `undeclared_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime rejects files written outside the declared output contract
- `timeout`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime enforces declared timeout budgets and records timeout failures
- `cancel`: Skip (enforced_by_runtime=false, advisory_only=true) - adapter descriptor does not declare cancellation support
- `env_policy`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime shapes and filters adapter environments before execution
- `workdir_isolation`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime executes process-backed adapters from a dedicated node work directory
- `missing_executable`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime reports executable resolution failures with structured infrastructure errors
- `cache_output`: Pass (enforced_by_runtime=true, advisory_only=false) - produced output schema matches the expected adapter schema
- `large_stdout`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime captures stdout and stderr as node evidence files
- `non_utf8_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime stores output bytes and artifact files without requiring UTF-8 payloads

### shell 0.1
- `success`: Pass (enforced_by_runtime=true, advisory_only=false) - successful adapter execution is a required runtime path
- `failure`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime records explicit execution failure results
- `argv_contract`: Pass (enforced_by_runtime=true, advisory_only=false) - shell nodes require a non-empty argv array of strings and a non-blank executable before execution starts
- `missing_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime validates declared output files for every adapter execution
- `undeclared_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime rejects files written outside the declared output contract
- `timeout`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime enforces declared timeout budgets and records timeout failures
- `cancel`: Skip (enforced_by_runtime=false, advisory_only=true) - adapter descriptor does not declare cancellation support
- `env_policy`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime shapes and filters adapter environments before execution
- `workdir_isolation`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime executes process-backed adapters from a dedicated node work directory
- `missing_executable`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime reports executable resolution failures with structured infrastructure errors
- `cache_output`: Pass (enforced_by_runtime=true, advisory_only=false) - produced output schema matches the expected adapter schema
- `large_stdout`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime captures stdout and stderr as node evidence files
- `non_utf8_output`: Pass (enforced_by_runtime=true, advisory_only=false) - runtime stores output bytes and artifact files without requiring UTF-8 payloads

## External adapter handshake boundary
- `info --json` must emit machine JSON on stdout only.
- non-empty stderr during the info handshake is rejected.
- external adapter binaries are fingerprinted into node trace evidence.

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
