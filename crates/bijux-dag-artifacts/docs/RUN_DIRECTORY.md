# Run Directory

The run directory is the durable evidence boundary for one DAG execution.
`RunDirLayout` computes paths; `RunDir` owns creation, resume, writes, and
finalization.

## Naming And Lifecycle

```text
run.tmp-abc/   # staging and resumable state
run-abc/       # finalized evidence
```

Run identifiers accept ASCII alphanumeric characters, hyphen, and underscore.
Empty identifiers, separators, and traversal segments are rejected.

Creation refuses existing staging or final paths. Resume is valid only when
exactly one exists: staging continues directly; final is renamed to staging;
neither is missing state; both is ambiguous state. Finalization renames
staging to final and never merges runs.

## Canonical Layout

Run records include `manifest.json`, `graph.snapshot.json`, `provenance.json`,
`run.log.jsonl`, `run.stop-request.json`, `outputs/index.json`, and
`nodes/<node-id>/`.

Node directories may contain input/output indexes, work and temporary
directories, stdout/stderr logs, trace, resolved params, attempt summaries, and
per-attempt streams. Callers use `RunDir` path methods instead of rebuilding
these locations.

## Atomic Records

Governed JSON records and graph snapshots use a sibling temporary file followed
by rename. This prevents partial final files, but finalization remains the
transaction boundary for the complete staged directory.

## Output Indexing

`write_outputs_index` requires normalized relative paths, existing outputs,
regular files or directories, checked size accounting, and computable SHA-256.
Entries are sorted by path. Missing or unsafe evidence is an error.

## Reader Rules

- Treat staging directories as incomplete unless explicitly resuming.
- Validate schema and manifest identity before trusting indexes.
- Resolve all paths beneath the run root and reject escapes.
- Do not infer success solely from a final directory.
- Preserve unknown future evidence during read-only inspection.

## Verification

```bash
cargo test --locked -p bijux-dag-artifacts --test io_store_fs_contracts
cargo test --locked -p bijux-dag-artifacts --test run_manifest_identity_contracts
cargo test --locked -p bijux-dag-artifacts --test run_manifest_roundtrip_and_retention_contracts
```
