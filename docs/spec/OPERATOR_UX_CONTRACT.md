# Operator UX Contract

## Personas
- local developer
- CI runner
- benchmark runner
- incident investigator
- release verifier

## Operator command classes
- run-time: `dag run`, `dag replay`
- inspect-time: `dag runs list|show|inspect|tree|timeline|diff|explain-failure|summary|compare|trend|failures|flakes`
- repair-time: `dag runs doctor`
- repo-time: `bijux-dev-dag repo run --domain governance`

## Stable operator run inspection surfaces
- `dag runs list --root <runs_dir>`
- `dag runs show <run_id> --root <runs_dir>`
- `dag runs inspect <run_id> --root <runs_dir>`
- `dag runs tree <run_id> --root <runs_dir>`
- `dag runs timeline <run_id> --root <runs_dir>`
- `dag runs diff <run_a_dir> <run_b_dir>`
- `dag runs verify <run_id> --root <runs_dir> [--deep]`
- `dag runs doctor <run_id> --root <runs_dir>`
- `dag runs explain-failure <run_id> --root <runs_dir>`
- `dag runs summary --root <runs_dir>`
- `dag runs compare <run_a> <run_b> --root <runs_dir>`
- `dag runs trend --root <runs_dir>`
- `dag runs failures --root <runs_dir>`
- `dag runs flakes --root <runs_dir>`

## Exit semantics
- `0`: command succeeded and reported healthy/valid result
- `3`: run data invalid/corrupt/missing for verify and doctor failure cases
- `2`: command usage/argument contract error
- `1`: internal error

## Output contracts
- Every run-inspection command supports `--json`.
- JSON schemas are in `configs/schema/operator/`.

## Corruption behavior
- inspection commands must return partial diagnostics when possible.
- verify and doctor must fail explicitly on invalid run state.

## Non-repo-coupled behavior
All `dag runs ...` commands operate on explicit `--root` and `run_id` inputs and
must not depend on ambient repository files.

## Command taxonomy ownership
Normative operator command taxonomy lives in:
- `docs/user/OPERATOR_COMMAND_INDEX.md`
- `docs/reference/COMMAND_TAXONOMY.md`

## Inspection contract ownership
- `docs/spec/OPERATOR_INSPECTION_CONTRACT.md`
- `docs/user/OPERATOR_INSPECTION_GUIDE.md`
