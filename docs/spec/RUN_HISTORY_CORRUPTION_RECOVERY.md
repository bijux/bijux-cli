# Run History Corruption Recovery

## Scope

This note defines the current recovery behavior for corrupted run-history directories consumed by `dag runs history`, `dag runs summary`, and `dag runs id-explain`.

## Current behavior

- Directory traversal is authoritative: each directory under the selected `--root` is treated as a run candidate.
- Corrupt `manifest.json` files are tolerated without panic; history rows are still emitted with `null`/fallback values.
- Analytics queries must not mutate authoritative run records.
- Alias artifacts such as `latest` must not rewrite or reorder historical run rows.

## Operator recovery steps

1. Run `dag runs history --root <runs_dir>` to enumerate all recoverable rows.
2. Run `dag runs doctor <run_id> --root <runs_dir>` for suspicious entries.
3. Rebuild or replace only corrupt run directories; do not rewrite healthy run manifests.
4. Re-run `dag runs summary --root <runs_dir>` to confirm recovered aggregate state.

## Non-goals

- Automatic rewriting of corrupted manifests.
- Silent deletion of corrupted run directories.
