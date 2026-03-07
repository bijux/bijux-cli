# App command ownership

## Responsibility inventory

`bijux-dag-app` owns command orchestration and must not own runtime internals.

- Parse CLI model into typed command requests.
- Execute command services that read DAG files, run directories, and export bundles.
- Render command results to JSON envelopes and human output.
- Convert domain and I/O failures into stable exit-code categories.

`bijux-dag-app` must not own:

- runtime scheduling or state-machine internals
- runtime policy merge primitives
- repository governance and release checks

## Service boundaries

- `commands/config_resolution.rs`
  - owns config precedence and policy precedence resolution
  - exposes typed request models for config and policy command paths
- `inspect/service.rs`
  - owns run inspection query execution for summary/tree/timeline/doctor/failure explain
- `replay/service.rs`
  - owns run-diff loading and semantic diff assembly

## Command flow diagram

```text
clap command model
      |
      v
app command router (lib.rs)
      |
      +--> config resolution service ----> effective runtime/policy model
      |
      +--> inspect service -------------> run inspection data
      |
      +--> replay service --------------> semantic run diff
      |
      v
result rendering (json envelope / human formatter)
```

## Ownership enforcement

- `app_architecture_contract` test blocks direct use of low-level config merge and direct run-diff assembly in `lib.rs`.
- `lib.rs` imports helper modules; it does not use source `include!` for graph helper injection.

