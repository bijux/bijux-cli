# Error Taxonomy

## Scope
Defines the unified error category model across `bijux-dag-core`, `bijux-dag-runtime`, `bijux-dag-app`, `bijux-dag-cli`, and `bijux-dev-dag`.

## Categories
- `parse`
- `schema`
- `validation`
- `config`
- `policy`
- `execution`
- `io`
- `replay`
- `cache`
- `compatibility`
- `internal`

## Mapping intent
- Core parsing and structural checks map to `parse` and `schema`.
- Semantic DAG rules map to `validation`.
- Runtime policy denials map to `policy`.
- Adapter/process failures map to `execution`.
- Storage and filesystem failures map to `io`.
- Replay and cache contract mismatches map to `replay` and `cache`.

## Diagnostic ordering policy
User-facing diagnostic ordering is stable for deterministic inputs. New diagnostics append by stable sort key (`category`, `code`, `path`).

## Related tests
- `crates/bijux-dag-app/tests/error_output_contract.rs`
- `crates/bijux-dag-app/tests/error_exit_contract.rs`

## Versioning and change policy
Adding categories is breaking unless consumers are proven category-agnostic. Category meaning changes require docs and snapshot updates in the same change.
